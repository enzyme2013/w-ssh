use anyhow::{anyhow, bail, Context, Result};
use dashmap::DashMap;
use russh::{client, ChannelMsg};
use russh_keys::key::{KeyPair, PublicKey};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};
use zeroize::Zeroize;

use crate::credentials::CredentialService;
use crate::host_trust::{HostTrustState, HostTrustStore};
use crate::models::{AuthMethod, ConnectSecret, Session};

pub struct TerminalHandle {
    pub write_tx: mpsc::Sender<Vec<u8>>,
    pub resize_tx: mpsc::Sender<(u32, u32)>,
}

pub type TerminalMap = Arc<DashMap<String, TerminalHandle>>;

struct TrustedHostHandler {
    trust: Arc<HostTrustStore>,
    host: String,
    port: u16,
}

#[async_trait::async_trait]
impl client::Handler for TrustedHostHandler {
    type Error = anyhow::Error;

    async fn check_server_key(&mut self, key: &PublicKey) -> Result<bool, Self::Error> {
        let result = self.trust.inspect(&self.host, self.port, key).await?;
        match result.state {
            HostTrustState::Trusted => Ok(true),
            HostTrustState::Unknown => bail!("HOST_KEY_UNKNOWN: 请先确认主机指纹"),
            HostTrustState::Changed => bail!("HOST_KEY_CHANGED: 主机密钥已变化，连接已阻止"),
        }
    }
}

struct ProbeHandler {
    captured: Arc<Mutex<Option<PublicKey>>>,
}

#[async_trait::async_trait]
impl client::Handler for ProbeHandler {
    type Error = anyhow::Error;

    async fn check_server_key(&mut self, key: &PublicKey) -> Result<bool, Self::Error> {
        *self.captured.lock().await = Some(key.clone());
        Ok(false)
    }
}

fn client_config() -> Arc<client::Config> {
    let config = client::Config {
        inactivity_timeout: Some(Duration::from_secs(20)),
        ..Default::default()
    };
    Arc::new(config)
}

pub async fn probe_host_key(
    trust: Arc<HostTrustStore>,
    host: String,
    port: u16,
) -> Result<crate::host_trust::HostTrustResult> {
    let captured = Arc::new(Mutex::new(None));
    let handler = ProbeHandler {
        captured: captured.clone(),
    };
    let _ = client::connect(client_config(), (host.as_str(), port), handler).await;
    let key = captured
        .lock()
        .await
        .take()
        .ok_or_else(|| anyhow!("HOST_KEY_UNAVAILABLE: 未能取得主机密钥"))?;
    trust.inspect(&host, port, &key).await
}

pub struct ConnectRequest {
    pub app_handle: AppHandle,
    pub terminal_id: String,
    pub terminal_map: TerminalMap,
    pub trust: Arc<HostTrustStore>,
    pub credentials: Arc<CredentialService>,
    pub session: Session,
    pub one_time_secret: Option<ConnectSecret>,
    pub cols: u32,
    pub rows: u32,
}

pub async fn connect(request: ConnectRequest) -> Result<()> {
    let ConnectRequest {
        app_handle,
        terminal_id,
        terminal_map,
        trust,
        credentials,
        session,
        one_time_secret,
        cols,
        rows,
    } = request;
    let host = session.host.clone();
    let port = u16::try_from(session.port).context("SSH 端口无效")?;
    let handler = TrustedHostHandler {
        trust,
        host: host.clone(),
        port,
    };
    let mut handle = client::connect(client_config(), (host.as_str(), port), handler)
        .await
        .map_err(|error| anyhow!("SSH 传输建立失败: {error}"))?;

    // Host verification has completed at this point. Secret retrieval must stay below this line.
    let mut secret = credentials
        .resolve_after_host_check(&session, one_time_secret)
        .await?;
    let auth_result = authenticate(&mut handle, &session, secret.as_deref()).await;
    if let Some(value) = secret.as_mut() {
        value.zeroize();
    }
    let auth_ok = auth_result?;
    if !auth_ok {
        bail!("SSH 认证未通过");
    }

    let mut channel = handle.channel_open_session().await?;
    channel
        .request_pty(false, "xterm-256color", cols, rows, 0, 0, &[])
        .await?;
    channel.request_shell(false).await?;

    let (write_tx, mut write_rx) = mpsc::channel::<Vec<u8>>(256);
    let (resize_tx, mut resize_rx) = mpsc::channel::<(u32, u32)>(32);
    terminal_map.insert(
        terminal_id.clone(),
        TerminalHandle {
            write_tx,
            resize_tx,
        },
    );

    let tid = terminal_id.clone();
    let ah = app_handle.clone();
    tokio::spawn(async move {
        let _handle = handle;
        loop {
            tokio::select! {
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { ref data }) => {
                            let _ = ah.emit(&format!("ssh_data_{tid}"), data.to_vec());
                        }
                        Some(ChannelMsg::ExitStatus { .. }) | Some(ChannelMsg::Eof) | None => break,
                        _ => {}
                    }
                }
                Some(data) = write_rx.recv() => {
                    if channel.data(data.as_ref()).await.is_err() { break; }
                }
                Some((cols, rows)) = resize_rx.recv() => {
                    let _ = channel.window_change(cols, rows, 0, 0).await;
                }
                else => break,
            }
        }
        terminal_map.remove(&tid);
        let _ = ah.emit(&format!("ssh_closed_{tid}"), ());
    });
    Ok(())
}

async fn authenticate(
    handle: &mut client::Handle<TrustedHostHandler>,
    session: &Session,
    secret: Option<&str>,
) -> Result<bool> {
    match session.auth_method {
        AuthMethod::Password => {
            let password = secret.ok_or_else(|| anyhow!("需要本次连接密码或已保存的系统凭据"))?;
            handle
                .authenticate_password(session.username.clone(), password)
                .await
                .map_err(|_| anyhow!("SSH 密码认证失败"))
        }
        AuthMethod::PrivateKey => {
            let key_path = session
                .private_key
                .as_deref()
                .ok_or_else(|| anyhow!("私钥认证缺少密钥路径"))?;
            let key_path = expand_home_path(key_path);
            let mut key_content = std::fs::read_to_string(&key_path)
                .map_err(|_| anyhow!("无法读取私钥文件: {}", key_path.display()))?;
            let decoded = russh_keys::decode_secret_key(&key_content, secret)
                .map_err(|_| anyhow!("无法解析私钥；如已加密请提供 passphrase"));
            key_content.zeroize();
            let key: KeyPair = decoded?;
            handle
                .authenticate_publickey(session.username.clone(), Arc::new(key))
                .await
                .map_err(|_| anyhow!("SSH 私钥认证失败"))
        }
    }
}

fn expand_home_path(path: &str) -> PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from);
    expand_home_path_with(path, home.as_deref())
}

fn expand_home_path_with(path: &str, home: Option<&Path>) -> PathBuf {
    let remainder = path.strip_prefix("~/").or_else(|| path.strip_prefix("~\\"));
    match (remainder, home) {
        (Some(remainder), Some(home)) => home.join(remainder),
        _ => PathBuf::from(path),
    }
}

#[cfg(test)]
mod path_tests {
    use super::expand_home_path_with;
    use std::path::Path;

    #[test]
    fn expands_portable_ssh_key_paths() {
        let home = Path::new("C:\\Users\\fixture");
        assert_eq!(
            expand_home_path_with("~/.ssh/id_ed25519", Some(home)),
            home.join(".ssh/id_ed25519")
        );
        assert_eq!(
            expand_home_path_with("C:\\keys\\custom", Some(home)),
            Path::new("C:\\keys\\custom")
        );
    }
}
