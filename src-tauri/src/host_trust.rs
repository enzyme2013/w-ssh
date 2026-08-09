use anyhow::{anyhow, bail, Context, Result};
use atomic_write_file::AtomicWriteFile;
use dashmap::DashMap;
use fs2::FileExt;
use russh_keys::key::PublicKey;
use russh_keys::PublicKeyBase64;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

const MAX_KNOWN_HOSTS_BYTES: u64 = 1024 * 1024;
const CHALLENGE_TTL: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostTrustState {
    Trusted,
    Unknown,
    Changed,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostTrustResult {
    pub state: HostTrustState,
    pub endpoint: String,
    pub algorithm: String,
    pub fingerprint: String,
    pub previous_fingerprints: Vec<String>,
    pub challenge_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HostTrustEntry {
    pub endpoint: String,
    pub algorithm: String,
    pub fingerprint: String,
}

#[derive(Clone)]
struct StoredKey {
    endpoint: String,
    algorithm: String,
    key: PublicKey,
}

struct PendingTrust {
    endpoint: String,
    key: PublicKey,
    previous_keys: Vec<PublicKey>,
    state: HostTrustState,
    created_at: Instant,
}

pub struct HostTrustStore {
    path: PathBuf,
    gate: Mutex<()>,
    pending: DashMap<String, PendingTrust>,
}

impl HostTrustStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            gate: Mutex::new(()),
            pending: DashMap::new(),
        }
    }

    pub async fn inspect(&self, host: &str, port: u16, key: &PublicKey) -> Result<HostTrustResult> {
        let endpoint = normalize_endpoint(host, port)?;
        let _guard = self.gate.lock().await;
        let entries = read_entries(&self.path)?;
        let matching_endpoint: Vec<_> = entries
            .iter()
            .filter(|entry| entry.endpoint == endpoint)
            .collect();
        let algorithm = canonical_algorithm(key)?.to_owned();
        let fingerprint = format!("SHA256:{}", key.fingerprint());

        if matching_endpoint.iter().any(|entry| entry.key == *key) {
            return Ok(HostTrustResult {
                state: HostTrustState::Trusted,
                endpoint,
                algorithm,
                fingerprint,
                previous_fingerprints: vec![],
                challenge_id: None,
            });
        }

        let state = if matching_endpoint.is_empty() {
            HostTrustState::Unknown
        } else {
            HostTrustState::Changed
        };
        let previous_fingerprints = matching_endpoint
            .iter()
            .map(|entry| format!("SHA256:{}", entry.key.fingerprint()))
            .collect();
        let previous_keys = matching_endpoint
            .iter()
            .map(|entry| entry.key.clone())
            .collect();
        self.pending
            .retain(|_, item| item.created_at.elapsed() < CHALLENGE_TTL);
        let challenge_id = Uuid::new_v4().to_string();
        self.pending.insert(
            challenge_id.clone(),
            PendingTrust {
                endpoint: endpoint.clone(),
                key: key.clone(),
                previous_keys,
                state,
                created_at: Instant::now(),
            },
        );
        Ok(HostTrustResult {
            state,
            endpoint,
            algorithm,
            fingerprint,
            previous_fingerprints,
            challenge_id: Some(challenge_id),
        })
    }

    pub async fn accept(&self, challenge_id: &str, replace: bool) -> Result<HostTrustEntry> {
        let (_, pending) = self
            .pending
            .remove(challenge_id)
            .ok_or_else(|| anyhow!("主机指纹确认已失效，请重新检查"))?;
        if pending.created_at.elapsed() >= CHALLENGE_TTL {
            bail!("主机指纹确认已过期，请重新检查");
        }
        match (pending.state, replace) {
            (HostTrustState::Unknown, false) | (HostTrustState::Changed, true) => {}
            (HostTrustState::Changed, false) => bail!("主机密钥已变化，必须使用独立替换流程"),
            (HostTrustState::Unknown, true) => bail!("首次信任不能使用替换流程"),
            (HostTrustState::Trusted, _) => bail!("已信任主机不需要确认"),
        }

        let _guard = self.gate.lock().await;
        let path = self.path.clone();
        let endpoint = pending.endpoint.clone();
        let key = pending.key.clone();
        let previous_keys = pending.previous_keys.clone();
        tokio::task::spawn_blocking(move || {
            update_entries(&path, &endpoint, key, replace, &previous_keys)
        })
        .await
        .context("known_hosts 写入任务失败")??;
        Ok(entry_to_public(StoredKey {
            endpoint: pending.endpoint,
            algorithm: canonical_algorithm(&pending.key)?.to_owned(),
            key: pending.key,
        }))
    }

    pub async fn list(&self) -> Result<Vec<HostTrustEntry>> {
        let _guard = self.gate.lock().await;
        Ok(read_entries(&self.path)?
            .into_iter()
            .map(entry_to_public)
            .collect())
    }

    pub async fn delete(&self, endpoint: &str) -> Result<bool> {
        if endpoint.trim().is_empty() || endpoint.contains(char::is_whitespace) {
            bail!("主机端点无效");
        }
        let endpoint = endpoint.to_owned();
        let _guard = self.gate.lock().await;
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || remove_endpoint(&path, &endpoint))
            .await
            .context("known_hosts 删除任务失败")?
    }
}

fn entry_to_public(entry: StoredKey) -> HostTrustEntry {
    HostTrustEntry {
        endpoint: entry.endpoint,
        algorithm: entry.algorithm,
        fingerprint: format!("SHA256:{}", entry.key.fingerprint()),
    }
}

pub fn normalize_endpoint(host: &str, port: u16) -> Result<String> {
    if port == 0 {
        bail!("SSH 端口必须在 1..=65535");
    }
    let trimmed = host.trim();
    if trimmed.is_empty()
        || trimmed.contains(char::is_whitespace)
        || trimmed.contains('\0')
        || trimmed.contains(',')
        || trimmed.contains('*')
        || trimmed.contains('?')
    {
        bail!("主机地址无效");
    }
    let unwrapped = match (trimmed.strip_prefix('['), trimmed.strip_suffix(']')) {
        (Some(_), None) | (None, Some(_)) => bail!("主机地址括号不完整"),
        (Some(value), Some(_)) => &value[..value.len() - 1],
        (None, None) => trimmed,
    };
    let normalized = match unwrapped.parse::<IpAddr>() {
        Ok(address) => address.to_string(),
        Err(_) => idna::domain_to_ascii(unwrapped)
            .map_err(|_| anyhow!("主机名不是有效 IDNA 域名"))?
            .trim_end_matches('.')
            .to_ascii_lowercase(),
    };
    if normalized.is_empty() {
        bail!("主机地址无效");
    }
    Ok(if port == 22 {
        normalized
    } else {
        format!("[{normalized}]:{port}")
    })
}

fn canonical_algorithm(key: &PublicKey) -> Result<&'static str> {
    Ok(match key {
        PublicKey::Ed25519(_) => "ssh-ed25519",
        PublicKey::RSA { .. } => "ssh-rsa",
        PublicKey::EC { key } => match key.algorithm() {
            "ecdsa-sha2-nistp256" => "ecdsa-sha2-nistp256",
            "ecdsa-sha2-nistp384" => "ecdsa-sha2-nistp384",
            "ecdsa-sha2-nistp521" => "ecdsa-sha2-nistp521",
            _ => bail!("不支持的主机密钥算法"),
        },
    })
}

fn read_entries(path: &Path) -> Result<Vec<StoredKey>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_KNOWN_HOSTS_BYTES {
        bail!("known_hosts 超过 1 MiB 安全上限");
    }
    let raw = std::fs::read_to_string(path).context("known_hosts 不是有效 UTF-8")?;
    let mut entries = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 3 || fields[0].starts_with('@') || fields[0].starts_with('|') {
            bail!("known_hosts 第 {} 行格式不受支持", index + 1);
        }
        if fields[0].contains(',') || fields[0].contains('*') || fields[0].contains('?') {
            bail!("known_hosts 第 {} 行包含不受支持的主机模式", index + 1);
        }
        let key = russh_keys::parse_public_key_base64(fields[2])
            .with_context(|| format!("known_hosts 第 {} 行公钥损坏", index + 1))?;
        let canonical = canonical_algorithm(&key)?;
        if fields[1] != canonical {
            bail!("known_hosts 第 {} 行算法与公钥不匹配", index + 1);
        }
        entries.push(StoredKey {
            endpoint: normalize_file_endpoint(fields[0])
                .with_context(|| format!("known_hosts 第 {} 行端点无效", index + 1))?,
            algorithm: canonical.to_owned(),
            key,
        });
    }
    Ok(entries)
}

fn normalize_file_endpoint(value: &str) -> Result<String> {
    if let Some(rest) = value.strip_prefix('[') {
        let (host, port) = rest
            .rsplit_once("]:")
            .ok_or_else(|| anyhow!("非默认端口必须使用 [host]:port"))?;
        let port = port.parse::<u16>().context("端口无效")?;
        return normalize_endpoint(host, port);
    }
    normalize_endpoint(value, 22)
}

fn update_entries(
    path: &Path,
    endpoint: &str,
    key: PublicKey,
    replace: bool,
    expected_previous: &[PublicKey],
) -> Result<()> {
    with_file_lock(path, || {
        let mut entries = read_entries(path)?;
        let current: Vec<_> = entries
            .iter()
            .filter(|entry| entry.endpoint == endpoint)
            .map(|entry| entry.key.clone())
            .collect();
        if current != expected_previous {
            bail!("主机信任状态已由其他进程修改，请重新检查");
        }
        if replace {
            entries.retain(|entry| entry.endpoint != endpoint);
        }
        entries.push(StoredKey {
            endpoint: endpoint.to_owned(),
            algorithm: canonical_algorithm(&key)?.to_owned(),
            key,
        });
        persist_entries(path, entries)
    })
}

fn remove_endpoint(path: &Path, endpoint: &str) -> Result<bool> {
    with_file_lock(path, || {
        let mut entries = read_entries(path)?;
        let previous = entries.len();
        entries.retain(|entry| entry.endpoint != endpoint);
        if previous == entries.len() {
            return Ok(false);
        }
        persist_entries(path, entries)?;
        Ok(true)
    })
}

fn persist_entries(path: &Path, mut entries: Vec<StoredKey>) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    entries.sort_by(|a, b| {
        a.endpoint
            .cmp(&b.endpoint)
            .then(a.algorithm.cmp(&b.algorithm))
    });
    let mut output = String::new();
    for entry in entries {
        output.push_str(&format!(
            "{} {} {}\n",
            entry.endpoint,
            entry.algorithm,
            entry.key.public_key_base64()
        ));
    }
    let mut file = AtomicWriteFile::open(path).context("无法创建 known_hosts 同目录临时文件")?;
    file.write_all(output.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    file.commit().context("无法原子替换 known_hosts")?;
    Ok(())
}

fn with_file_lock<T>(path: &Path, operation: impl FnOnce() -> Result<T>) -> Result<T> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("known_hosts 文件名不是有效 UTF-8"))?;
    let lock_path = path.with_file_name(format!("{file_name}.lock"));
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    lock.lock_exclusive().context("无法锁定 known_hosts")?;
    let result = operation();
    let unlock_result = FileExt::unlock(&lock).context("无法解锁 known_hosts");
    match result {
        Ok(value) => {
            unlock_result?;
            Ok(value)
        }
        Err(error) => {
            let _ = unlock_result;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use russh_keys::key::KeyPair;
    use russh_keys::key::SignatureHash;
    use std::sync::Arc;

    fn fixture() -> (PathBuf, Arc<HostTrustStore>, PublicKey, PublicKey) {
        let root = std::env::temp_dir().join(format!("w-ssh-known-hosts-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let store = Arc::new(HostTrustStore::new(root.join("known_hosts")));
        let first = KeyPair::generate_ed25519().clone_public_key().unwrap();
        let second = KeyPair::generate_ed25519().clone_public_key().unwrap();
        (root, store, first, second)
    }

    #[test]
    fn normalizes_dns_idna_ipv4_and_ipv6_endpoints() {
        assert_eq!(
            normalize_endpoint("EXAMPLE.COM.", 22).unwrap(),
            "example.com"
        );
        assert_eq!(
            normalize_endpoint("bücher.example", 2222).unwrap(),
            "[xn--bcher-kva.example]:2222"
        );
        assert_eq!(
            normalize_endpoint("[2001:0db8::1]", 22).unwrap(),
            "2001:db8::1"
        );
        assert_eq!(
            normalize_endpoint("2001:db8::1", 2222).unwrap(),
            "[2001:db8::1]:2222"
        );
    }

    #[tokio::test]
    async fn tofu_match_and_changed_key_are_distinct_and_fail_closed() {
        let (root, store, first, second) = fixture();
        let unknown = store.inspect("Example.test", 22, &first).await.unwrap();
        assert_eq!(unknown.state, HostTrustState::Unknown);
        store
            .accept(unknown.challenge_id.as_deref().unwrap(), false)
            .await
            .unwrap();
        assert_eq!(
            store
                .inspect("example.test", 22, &first)
                .await
                .unwrap()
                .state,
            HostTrustState::Trusted
        );
        let changed = store.inspect("example.test", 22, &second).await.unwrap();
        assert_eq!(changed.state, HostTrustState::Changed);
        assert!(store
            .accept(changed.challenge_id.as_deref().unwrap(), false)
            .await
            .is_err());
        let changed = store.inspect("example.test", 22, &second).await.unwrap();
        store
            .accept(changed.challenge_id.as_deref().unwrap(), true)
            .await
            .unwrap();
        assert_eq!(
            store
                .inspect("example.test", 22, &second)
                .await
                .unwrap()
                .state,
            HostTrustState::Trusted
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn corrupt_file_blocks_read_and_write() {
        let (root, store, key, _) = fixture();
        std::fs::write(root.join("known_hosts"), "bad line\n").unwrap();
        assert!(store.inspect("example.test", 22, &key).await.is_err());
        assert!(store.list().await.is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn concurrent_first_writers_do_not_lose_or_overwrite_trust() {
        let (root, first_store, first, _) = fixture();
        let second_store = Arc::new(HostTrustStore::new(root.join("known_hosts")));
        let other = KeyPair::generate_ed25519().clone_public_key().unwrap();
        let a = first_store.inspect("a.test", 22, &first).await.unwrap();
        let b = second_store.inspect("b.test", 22, &other).await.unwrap();
        let (ra, rb) = tokio::join!(
            first_store.accept(a.challenge_id.as_deref().unwrap(), false),
            second_store.accept(b.challenge_id.as_deref().unwrap(), false)
        );
        ra.unwrap();
        rb.unwrap();
        assert_eq!(first_store.list().await.unwrap().len(), 2);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn rsa_signature_variant_uses_canonical_ssh_rsa_key_material() {
        let (root, store, _, _) = fixture();
        let pair = KeyPair::generate_rsa(2048, SignatureHash::SHA2_512).unwrap();
        let mut key = pair.clone_public_key().unwrap();
        let unknown = store.inspect("rsa.test", 22, &key).await.unwrap();
        assert_eq!(unknown.algorithm, "ssh-rsa");
        store
            .accept(unknown.challenge_id.as_deref().unwrap(), false)
            .await
            .unwrap();
        key.set_algorithm(SignatureHash::SHA2_256);
        assert_eq!(
            store.inspect("rsa.test", 22, &key).await.unwrap().state,
            HostTrustState::Trusted
        );
        assert!(std::fs::read_to_string(root.join("known_hosts"))
            .unwrap()
            .contains(" ssh-rsa "));
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn stale_replacement_challenge_cannot_overwrite_newer_trust() {
        let (root, first_store, first, second) = fixture();
        let unknown = first_store.inspect("race.test", 22, &first).await.unwrap();
        first_store
            .accept(unknown.challenge_id.as_deref().unwrap(), false)
            .await
            .unwrap();
        let stale = first_store.inspect("race.test", 22, &second).await.unwrap();

        let newer_store = HostTrustStore::new(root.join("known_hosts"));
        let third = KeyPair::generate_ed25519().clone_public_key().unwrap();
        let newer = newer_store.inspect("race.test", 22, &third).await.unwrap();
        newer_store
            .accept(newer.challenge_id.as_deref().unwrap(), true)
            .await
            .unwrap();

        assert!(first_store
            .accept(stale.challenge_id.as_deref().unwrap(), true)
            .await
            .is_err());
        assert_eq!(
            first_store
                .inspect("race.test", 22, &third)
                .await
                .unwrap()
                .state,
            HostTrustState::Trusted
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
