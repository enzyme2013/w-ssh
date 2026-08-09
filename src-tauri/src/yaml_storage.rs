use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_saphyr::granit_parser::{Event, Parser};
use serde_saphyr::options::MergeKeyPolicy;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::{
    AuthMethod, CreateSession, CredentialState, Session, UpdateGroup, UpdateSession,
};
use crate::storage::{now_str, SessionStorage};

const SCHEMA_VERSION: u32 = 1;
const MAX_YAML_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct YamlDocument {
    schema_version: u32,
    sessions: Vec<YamlSession>,
}

impl Default for YamlDocument {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            sessions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct YamlSession {
    id: String,
    name: String,
    host: String,
    port: i64,
    username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    group_icon: Option<String>,
    auth: YamlAuth,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct YamlAuth {
    method: YamlAuthMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    private_key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum YamlAuthMethod {
    Password,
    PrivateKey,
}

pub struct YamlSessionStorage {
    path: PathBuf,
    gate: Mutex<()>,
    notice: Mutex<Option<String>>,
}

impl YamlSessionStorage {
    pub async fn open(path: PathBuf) -> Result<Self> {
        validate_yaml_path(&path)?;
        let open_path = path.clone();
        tokio::task::spawn_blocking(move || load_or_create(&open_path))
            .await
            .context("YAML initialization task failed")??;

        Ok(Self {
            path,
            gate: Mutex::new(()),
            notice: Mutex::new(None),
        })
    }

    async fn read_document(&self) -> Result<YamlDocument> {
        let _guard = self.gate.lock().await;
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || load_or_create(&path).map(|(doc, _)| doc))
            .await
            .context("YAML read task failed")?
    }

    async fn mutate<T, F>(&self, change: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut YamlDocument) -> Result<(T, bool)> + Send + 'static,
    {
        let _guard = self.gate.lock().await;
        let path = self.path.clone();
        let (value, recovery) =
            tokio::task::spawn_blocking(move || -> Result<(T, Option<PathBuf>)> {
                let (mut document, raw) = load_or_create(&path)?;
                let (value, changed) = change(&mut document)?;
                if !changed {
                    return Ok((value, None));
                }
                validate_document(&document)?;
                let recovery = persist_document(&path, Some(&raw), &document)?;
                Ok((value, recovery))
            })
            .await
            .context("YAML write task failed")??;

        if let Some(path) = recovery {
            *self.notice.lock().await = Some(format!(
                "YAML 已规范化；原始文件恢复副本：{}",
                path.display()
            ));
        }
        Ok(value)
    }
}

#[async_trait]
impl SessionStorage for YamlSessionStorage {
    async fn list(&self) -> Result<Vec<Session>> {
        let mut sessions: Vec<_> = self
            .read_document()
            .await?
            .sessions
            .into_iter()
            .map(Session::from)
            .collect();
        sort_sessions(&mut sessions);
        Ok(sessions)
    }

    async fn get(&self, id: &str) -> Result<Session> {
        self.read_document()
            .await?
            .sessions
            .into_iter()
            .find(|session| session.id == id)
            .map(Session::from)
            .ok_or_else(|| anyhow!("会话不存在: {id}"))
    }

    async fn create(&self, data: CreateSession) -> Result<Session> {
        let session = yaml_session_from_create(data)?;
        let result = Session::from(session.clone());
        self.mutate(move |document| {
            document.sessions.push(session);
            Ok((result, true))
        })
        .await
    }

    async fn update(&self, data: UpdateSession) -> Result<Session> {
        let replacement = yaml_session_from_update(data)?;
        self.mutate(move |document| {
            let current = document
                .sessions
                .iter_mut()
                .find(|session| session.id == replacement.id)
                .ok_or_else(|| anyhow!("会话不存在: {}", replacement.id))?;
            let created_at = current.created_at.clone();
            *current = replacement;
            current.created_at = created_at;
            let result = Session::from(current.clone());
            Ok((result, true))
        })
        .await
    }

    async fn update_group(&self, data: UpdateGroup) -> Result<Vec<Session>> {
        let current_name = data.current_name.trim().to_owned();
        let name = data.name.trim().to_owned();
        validate_group(&current_name, &name, data.icon.as_deref())?;
        self.mutate(move |document| {
            if current_name != name
                && document
                    .sessions
                    .iter()
                    .any(|session| session.group_name.as_deref() == Some(name.as_str()))
            {
                bail!("目标分组已存在，请使用其他名称");
            }

            let now = now_str();
            let mut updated = Vec::new();
            for session in &mut document.sessions {
                if session.group_name.as_deref() == Some(current_name.as_str()) {
                    session.group_name = Some(name.clone());
                    session.group_icon = data.icon.clone();
                    session.updated_at = now.clone();
                    updated.push(Session::from(session.clone()));
                }
            }
            if updated.is_empty() {
                bail!("分组不存在: {current_name}");
            }
            Ok((updated, true))
        })
        .await
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let id = id.to_owned();
        self.mutate(move |document| {
            let previous = document.sessions.len();
            document.sessions.retain(|session| session.id != id);
            Ok(((), previous != document.sessions.len()))
        })
        .await
    }

    async fn import_if_empty(&self, sessions: Vec<Session>) -> Result<()> {
        let imported = sessions
            .into_iter()
            .map(yaml_session_from_session)
            .collect::<Result<Vec<_>>>()?;
        self.mutate(move |document| {
            if !document.sessions.is_empty() {
                bail!("YAML 目标存储非空，复制已取消");
            }
            document.sessions = imported;
            Ok(((), true))
        })
        .await
    }

    async fn take_notice(&self) -> Option<String> {
        self.notice.lock().await.take()
    }
}

fn validate_yaml_path(path: &Path) -> Result<()> {
    if path.extension().and_then(|value| value.to_str()) != Some("yml") {
        bail!("YAML 配置路径必须使用 .yml 扩展名");
    }
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("YAML 路径缺少父目录"))?;
    if !parent.exists() {
        bail!("YAML 配置父目录不存在: {}", parent.display());
    }
    Ok(())
}

fn load_or_create(path: &Path) -> Result<(YamlDocument, String)> {
    if !path.exists() {
        let document = YamlDocument::default();
        persist_document(path, None, &document)?;
    }

    let metadata = std::fs::metadata(path)
        .with_context(|| format!("无法读取 YAML 元数据: {}", path.display()))?;
    if metadata.len() > MAX_YAML_BYTES {
        bail!("YAML 配置超过 4 MiB 安全上限");
    }

    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("YAML 配置不是有效 UTF-8: {}", path.display()))?;
    let document = parse_document(&raw)?;
    Ok((document, raw))
}

fn parse_document(raw: &str) -> Result<YamlDocument> {
    reject_advanced_yaml(raw)?;
    let options = serde_saphyr::options! {
        strict_booleans: true,
        merge_keys: MergeKeyPolicy::Error,
    };
    let value: Value =
        serde_saphyr::from_str_with_options(raw, options.clone()).context("YAML 结构解析失败")?;
    reject_sensitive_keys(&value)?;
    let document: YamlDocument =
        serde_saphyr::from_str_with_options(raw, options).context("YAML schema v1 校验失败")?;
    validate_document(&document)?;
    Ok(document)
}

fn reject_advanced_yaml(raw: &str) -> Result<()> {
    for next in Parser::new_from_str(raw) {
        let (event, _) = next.context("YAML 语法解析失败")?;
        match event {
            Event::Alias(_) => bail!("YAML v1 不允许 alias"),
            Event::Scalar(_, _, anchor, tag)
            | Event::SequenceStart(_, anchor, tag)
            | Event::MappingStart(_, anchor, tag)
                if anchor != 0 || tag.is_some() =>
            {
                bail!("YAML v1 不允许 anchor 或显式 tag")
            }
            Event::DocumentStart(_, Some(version)) if version.major != 1 || version.minor != 2 => {
                bail!("YAML v1 只接受 YAML 1.2")
            }
            _ => {}
        }
    }
    Ok(())
}

fn reject_sensitive_keys(value: &Value) -> Result<()> {
    match value {
        Value::Object(values) => {
            for (key, value) in values {
                if matches!(key.as_str(), "password" | "private_key_content") {
                    bail!("YAML schema 禁止密码或私钥内容字段");
                }
                reject_sensitive_keys(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                reject_sensitive_keys(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_document(document: &YamlDocument) -> Result<()> {
    if document.schema_version != SCHEMA_VERSION {
        bail!("不支持的 YAML schema_version: {}", document.schema_version);
    }
    let mut ids = HashSet::new();
    for session in &document.sessions {
        validate_session(session)?;
        if !ids.insert(session.id.as_str()) {
            bail!("YAML 包含重复会话 ID: {}", session.id);
        }
    }
    Ok(())
}

fn validate_session(session: &YamlSession) -> Result<()> {
    for (label, value) in [
        ("id", session.id.as_str()),
        ("name", session.name.as_str()),
        ("host", session.host.as_str()),
        ("username", session.username.as_str()),
        ("created_at", session.created_at.as_str()),
        ("updated_at", session.updated_at.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!("YAML 会话字段 {label} 不能为空");
        }
        if value.contains('\0') {
            bail!("YAML 会话字段 {label} 包含非法字符");
        }
    }
    if !(1..=65_535).contains(&session.port) {
        bail!("YAML 会话端口必须在 1..=65535");
    }
    match session.auth.method {
        YamlAuthMethod::Password if session.auth.private_key.is_some() => {
            bail!("password 认证不能包含 private_key")
        }
        YamlAuthMethod::PrivateKey => {
            let path = session
                .auth
                .private_key
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| anyhow!("private_key 认证必须提供密钥路径"))?;
            if path.len() > 4096
                || path.contains('\n')
                || path.contains('\r')
                || path.contains("PRIVATE KEY")
            {
                bail!("private_key 只允许保存文件路径，不允许私钥内容");
            }
        }
        _ => {}
    }
    Ok(())
}

fn yaml_session_from_create(data: CreateSession) -> Result<YamlSession> {
    let now = now_str();
    let session = YamlSession {
        id: Uuid::new_v4().to_string(),
        name: data.name,
        host: data.host,
        port: data.port,
        username: data.username,
        icon: data.icon,
        group_name: data.group_name,
        group_icon: data.group_icon,
        auth: yaml_auth(data.auth_method, data.private_key)?,
        created_at: now.clone(),
        updated_at: now,
    };
    validate_session(&session)?;
    Ok(session)
}

fn yaml_session_from_update(data: UpdateSession) -> Result<YamlSession> {
    let session = YamlSession {
        id: data.id,
        name: data.name,
        host: data.host,
        port: data.port,
        username: data.username,
        icon: data.icon,
        group_name: data.group_name,
        group_icon: data.group_icon,
        auth: yaml_auth(data.auth_method, data.private_key)?,
        created_at: String::new(),
        updated_at: now_str(),
    };
    let mut validation = session.clone();
    validation.created_at = "preserved".into();
    validate_session(&validation)?;
    Ok(session)
}

fn yaml_session_from_session(session: Session) -> Result<YamlSession> {
    let imported = YamlSession {
        id: session.id,
        name: session.name,
        host: session.host,
        port: session.port,
        username: session.username,
        icon: session.icon,
        group_name: session.group_name,
        group_icon: session.group_icon,
        auth: yaml_auth(session.auth_method, session.private_key)?,
        created_at: session.created_at,
        updated_at: session.updated_at,
    };
    validate_session(&imported)?;
    Ok(imported)
}

fn yaml_auth(auth_method: AuthMethod, private_key: Option<String>) -> Result<YamlAuth> {
    match (auth_method, private_key) {
        (AuthMethod::PrivateKey, Some(path)) => Ok(YamlAuth {
            method: YamlAuthMethod::PrivateKey,
            private_key: Some(path),
        }),
        (AuthMethod::Password, None) => Ok(YamlAuth {
            method: YamlAuthMethod::Password,
            private_key: None,
        }),
        (AuthMethod::Password, Some(_)) => bail!("password 认证不能包含 private_key"),
        (AuthMethod::PrivateKey, None) => bail!("private_key 认证必须提供密钥路径"),
    }
}

fn serialize_document(document: &YamlDocument) -> Result<String> {
    let mut stable = document.clone();
    stable.sessions.sort_by(|a, b| {
        a.group_name
            .cmp(&b.group_name)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut output = serde_saphyr::to_string(&stable).context("YAML 序列化失败")?;
    if !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

fn persist_document(
    path: &Path,
    current_raw: Option<&str>,
    document: &YamlDocument,
) -> Result<Option<PathBuf>> {
    persist_document_with(
        path,
        current_raw,
        document,
        create_recovery_backup,
        |p, bytes| atomic_write_with_hook(p, bytes, || Ok(())),
    )
}

fn persist_document_with<B, W>(
    path: &Path,
    current_raw: Option<&str>,
    document: &YamlDocument,
    backup: B,
    write: W,
) -> Result<Option<PathBuf>>
where
    B: FnOnce(&Path, &[u8]) -> Result<PathBuf>,
    W: FnOnce(&Path, &[u8]) -> Result<()>,
{
    let output = serialize_document(document)?;
    let recovery = match current_raw {
        Some(raw) if raw != serialize_document(&parse_document(raw)?)? => {
            Some(backup(path, raw.as_bytes())?)
        }
        _ => None,
    };
    write(path, output.as_bytes())?;
    Ok(recovery)
}

fn create_recovery_backup(path: &Path, bytes: &[u8]) -> Result<PathBuf> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("YAML 文件名不是有效 UTF-8"))?;
    let backup = path.with_file_name(format!(
        "{file_name}.recovery-{}-{}.bak",
        now_str(),
        Uuid::new_v4()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup)
        .with_context(|| format!("无法创建 YAML 恢复副本: {}", backup.display()))?;
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;
    Ok(backup)
}

fn atomic_write_with_hook<F>(path: &Path, bytes: &[u8], before_commit: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    let mut file = AtomicWriteFile::open(path)
        .with_context(|| format!("无法创建 YAML 同目录临时文件: {}", path.display()))?;
    file.write_all(bytes)?;
    file.flush()?;
    file.sync_all()?;
    before_commit()?;
    file.commit()
        .with_context(|| format!("无法原子替换 YAML 文件: {}", path.display()))?;
    Ok(())
}

fn sort_sessions(sessions: &mut [Session]) {
    sessions.sort_by(|a, b| {
        a.group_name
            .cmp(&b.group_name)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.id.cmp(&b.id))
    });
}

impl From<YamlSession> for Session {
    fn from(value: YamlSession) -> Self {
        Self {
            id: value.id,
            name: value.name,
            host: value.host,
            port: value.port,
            username: value.username,
            private_key: value.auth.private_key,
            auth_method: match value.auth.method {
                YamlAuthMethod::Password => AuthMethod::Password,
                YamlAuthMethod::PrivateKey => AuthMethod::PrivateKey,
            },
            credential_state: CredentialState::None,
            icon: value.icon,
            group_name: value.group_name,
            group_icon: value.group_icon,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

fn validate_group(current_name: &str, name: &str, icon: Option<&str>) -> Result<()> {
    if current_name.is_empty() || name.is_empty() {
        bail!("分组名称不能为空");
    }
    if current_name.len() > 200
        || name.len() > 200
        || current_name.contains('\0')
        || name.contains('\0')
    {
        bail!("分组名称无效或过长");
    }
    if icon.is_some_and(|value| value.len() > 64 || value.contains('\0')) {
        bail!("分组图标无效");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::assert_storage_contract;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    fn temp_yaml(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("w-ssh-yaml-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(format!("{name}.yml"))
    }

    fn cleanup(path: &Path) {
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn yaml_implements_non_sensitive_storage_contract() {
        let path = temp_yaml("contract");
        let storage = YamlSessionStorage::open(path.clone()).await.unwrap();
        assert_storage_contract(&storage).await;
        cleanup(&path);
    }

    #[tokio::test]
    async fn restart_recovers_stable_data_without_passwords() {
        let path = temp_yaml("restart");
        let storage = YamlSessionStorage::open(path.clone()).await.unwrap();
        let created = storage
            .create(CreateSession {
                name: "Password host".into(),
                host: "example.test".into(),
                port: 22,
                username: "user".into(),
                private_key: None,
                auth_method: AuthMethod::Password,
                icon: None,
                group_name: None,
                group_icon: None,
            })
            .await
            .unwrap();
        assert_eq!(created.credential_state, CredentialState::None);
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("never-persist"));
        assert!(!raw.contains("password:"));

        drop(storage);
        let reopened = YamlSessionStorage::open(path.clone()).await.unwrap();
        assert_eq!(reopened.get(&created.id).await.unwrap(), created);
        cleanup(&path);
    }

    #[test]
    fn rejects_schema_corruption_duplicate_ids_and_invalid_ports() {
        for invalid in [
            "schema_version: 2\nsessions: []\n",
            "schema_version: 1\nsessions: [\n",
            "schema_version: 1\nsessions:\n  - id: same\n    name: a\n    host: a\n    port: 22\n    username: u\n    auth: { method: password }\n    created_at: '1'\n    updated_at: '1'\n  - id: same\n    name: b\n    host: b\n    port: 22\n    username: u\n    auth: { method: password }\n    created_at: '1'\n    updated_at: '1'\n",
            "schema_version: 1\nsessions:\n  - id: bad-port\n    name: bad\n    host: bad\n    port: 70000\n    username: u\n    auth: { method: password }\n    created_at: '1'\n    updated_at: '1'\n",
        ] {
            assert!(parse_document(invalid).is_err(), "accepted: {invalid}");
        }
    }

    #[test]
    fn rejects_advanced_yaml_and_secret_fields() {
        for invalid in [
            "%YAML 1.1\n---\nschema_version: 1\nsessions: []\n",
            "schema_version: 1\nsessions: &items []\n",
            "schema_version: 1\nsessions: &items []\ncopy: *items\n",
            "schema_version: 1\nsessions: !custom []\n",
            "schema_version: 1\nbase: &base { sessions: [] }\n<<: *base\n",
            "schema_version: 1\nsessions:\n  - id: secret\n    name: secret\n    host: host\n    port: 22\n    username: user\n    password: should-not-parse\n    auth: { method: password }\n    created_at: '1'\n    updated_at: '1'\n",
            "schema_version: 1\nsessions:\n  - id: embedded-key\n    name: embedded\n    host: host\n    port: 22\n    username: user\n    auth:\n      method: private_key\n      private_key: |\n        -----BEGIN PRIVATE KEY-----\n        fixture-only\n        -----END PRIVATE KEY-----\n    created_at: '1'\n    updated_at: '1'\n",
        ] {
            assert!(parse_document(invalid).is_err(), "accepted: {invalid}");
        }
    }

    #[test]
    fn atomic_failure_preserves_last_valid_file() {
        let path = temp_yaml("atomic-failure");
        let original = serialize_document(&YamlDocument::default()).unwrap();
        std::fs::write(&path, &original).unwrap();
        let replacement = YamlDocument {
            schema_version: 1,
            sessions: vec![],
        };
        let result = atomic_write_with_hook(
            &path,
            serialize_document(&replacement).unwrap().as_bytes(),
            || bail!("injected failure before commit"),
        );
        assert!(result.is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        assert!(parse_document(&std::fs::read_to_string(&path).unwrap()).is_ok());
        cleanup(&path);
    }

    #[test]
    fn comment_normalization_creates_exact_recovery_backup() {
        let path = temp_yaml("comments");
        let raw = "# user note\nschema_version: 1\nsessions: []\n";
        std::fs::write(&path, raw).unwrap();
        let document = parse_document(raw).unwrap();
        let recovery = persist_document(&path, Some(raw), &document)
            .unwrap()
            .unwrap();
        assert_eq!(std::fs::read_to_string(recovery).unwrap(), raw);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            serialize_document(&document).unwrap()
        );
        cleanup(&path);
    }

    #[test]
    fn backup_failure_aborts_before_target_write() {
        let path = temp_yaml("backup-failure");
        let raw = "# user note\nschema_version: 1\nsessions: []\n";
        std::fs::write(&path, raw).unwrap();
        let write_called = AtomicBool::new(false);
        let result = persist_document_with(
            &path,
            Some(raw),
            &parse_document(raw).unwrap(),
            |_, _| bail!("injected backup failure"),
            |_, _| {
                write_called.store(true, Ordering::SeqCst);
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(!write_called.load(Ordering::SeqCst));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), raw);
        cleanup(&path);
    }

    #[tokio::test]
    async fn concurrent_creates_are_serialized_without_loss() {
        let path = temp_yaml("concurrent");
        let storage = Arc::new(YamlSessionStorage::open(path.clone()).await.unwrap());
        let mut tasks = Vec::new();
        for index in 0..16 {
            let storage = storage.clone();
            tasks.push(tokio::spawn(async move {
                storage
                    .create(CreateSession {
                        name: format!("host-{index}"),
                        host: format!("host-{index}.test"),
                        port: 22,
                        username: "user".into(),
                        private_key: Some(format!("fixture-key-{index}")),
                        auth_method: AuthMethod::PrivateKey,
                        icon: None,
                        group_name: None,
                        group_icon: None,
                    })
                    .await
                    .unwrap();
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }
        assert_eq!(storage.list().await.unwrap().len(), 16);
        assert!(parse_document(&std::fs::read_to_string(&path).unwrap()).is_ok());
        cleanup(&path);
    }
}
