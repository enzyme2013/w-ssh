use std::sync::Arc;
use tokio::sync::mpsc;
use dashmap::DashMap;
use russh::{ChannelMsg, client};
use russh_keys::key::KeyPair;
use anyhow::{Result, anyhow};
use tauri::{AppHandle, Emitter};

/// 每个终端标签对应的控制句柄
pub struct TerminalHandle {
    pub write_tx: mpsc::Sender<Vec<u8>>,
    pub resize_tx: mpsc::Sender<(u32, u32)>,
}

pub type TerminalMap = Arc<DashMap<String, TerminalHandle>>;

/// 简单的 client handler，MVP 阶段接受所有服务端 key
struct ClientHandler;

#[async_trait::async_trait]
impl client::Handler for ClientHandler {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh_keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

/// 建立 SSH 连接并启动后台 IO 任务
pub async fn connect(
    app_handle: AppHandle,
    terminal_id: String,
    terminal_map: TerminalMap,
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key: Option<String>,
    cols: u32,
    rows: u32,
) -> Result<()> {
    let config = Arc::new(client::Config::default());

    let mut handle = client::connect(config, (host.as_str(), port), ClientHandler)
        .await
        .map_err(|e| anyhow!("连接失败: {}", e))?;

    // 认证
    let auth_ok = if let Some(pwd) = password {
        handle.authenticate_password(username.clone(), pwd).await?
    } else if let Some(key_path) = private_key {
        let key_content = std::fs::read_to_string(&key_path)
            .map_err(|e| anyhow!("读取私钥文件 '{}' 失败: {}", key_path, e))?;
        let key: KeyPair = russh_keys::decode_secret_key(&key_content, None)
            .map_err(|e| anyhow!("解析私钥失败: {}", e))?;
        handle
            .authenticate_publickey(username.clone(), Arc::new(key))
            .await?
    } else {
        return Err(anyhow!("未提供认证方式"));
    };

    if !auth_ok {
        return Err(anyhow!("认证失败，请检查用户名和密码"));
    }

    // 打开 shell channel
    let mut channel = handle.channel_open_session().await?;
    channel
        .request_pty(false, "xterm-256color", cols, rows, 0, 0, &[])
        .await?;
    channel.request_shell(false).await?;

    // 用于前端 → SSH 的写入和 resize 通道
    let (write_tx, mut write_rx) = mpsc::channel::<Vec<u8>>(256);
    let (resize_tx, mut resize_rx) = mpsc::channel::<(u32, u32)>(32);

    terminal_map.insert(
        terminal_id.clone(),
        TerminalHandle { write_tx, resize_tx },
    );

    let tid = terminal_id.clone();
    let ah = app_handle.clone();

    tokio::spawn(async move {
        // 保持 handle 存活
        let _handle = handle;

        loop {
            tokio::select! {
                // SSH server 输出 → 前端 xterm
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { ref data }) => {
                            let bytes: Vec<u8> = data.to_vec();
                            let _ = ah.emit(&format!("ssh_data_{}", tid), bytes);
                        }
                        Some(ChannelMsg::ExitStatus { .. })
                        | Some(ChannelMsg::Eof)
                        | None => {
                            break;
                        }
                        _ => {}
                    }
                }
                // 前端 xterm 输入 → SSH server
                Some(data) = write_rx.recv() => {
                    if channel.data(data.as_ref()).await.is_err() {
                        break;
                    }
                }
                // 窗口大小变化
                Some((c, r)) = resize_rx.recv() => {
                    let _ = channel.window_change(c, r, 0, 0).await;
                }
                else => break,
            }
        }

        terminal_map.remove(&tid);
        let _ = ah.emit(&format!("ssh_closed_{}", tid), ());
    });

    Ok(())
}
