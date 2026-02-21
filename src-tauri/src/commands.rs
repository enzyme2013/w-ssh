use tauri::{AppHandle, State};
use crate::{db, models::*, ssh, AppState};

type CmdResult<T> = Result<T, String>;

fn to_str<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// ── 会话管理 ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_sessions(state: State<'_, AppState>) -> CmdResult<Vec<Session>> {
    db::get_sessions(&state.db).await.map_err(to_str)
}

#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    data: CreateSession,
) -> CmdResult<Session> {
    db::create_session(&state.db, data).await.map_err(to_str)
}

#[tauri::command]
pub async fn update_session(
    state: State<'_, AppState>,
    data: UpdateSession,
) -> CmdResult<Session> {
    db::update_session(&state.db, data).await.map_err(to_str)
}

#[tauri::command]
pub async fn delete_session(
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    db::delete_session(&state.db, &id).await.map_err(to_str)
}

// ── 工具 ────────────────────────────────────────────────────────────────────

/// 扫描 ~/.ssh/ 下的常见私钥文件，返回存在的绝对路径列表
#[tauri::command]
pub fn get_ssh_key_paths() -> Vec<String> {
    let common_names = ["id_rsa", "id_ed25519", "id_ecdsa", "id_dsa", "id_ecdsa_sk", "id_ed25519_sk"];

    // 兼容 Windows（USERPROFILE）和 Unix（HOME）
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();

    if home.is_empty() {
        return vec![];
    }

    let ssh_dir = std::path::Path::new(&home).join(".ssh");
    common_names
        .iter()
        .map(|name| ssh_dir.join(name))
        .filter(|p| p.exists())
        .filter_map(|p| p.to_str().map(|s| s.to_string()))
        .collect()
}

// ── SSH ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn ssh_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    cols: u32,
    rows: u32,
) -> CmdResult<String> {
    let session = db::get_session_by_id(&state.db, &session_id)
        .await
        .map_err(to_str)?;

    let terminal_id = uuid::Uuid::new_v4().to_string();

    ssh::connect(
        app,
        terminal_id.clone(),
        state.terminals.clone(),
        session.host,
        session.port as u16,
        session.username,
        session.password,
        session.private_key,
        cols,
        rows,
    )
    .await
    .map_err(to_str)?;

    Ok(terminal_id)
}

#[tauri::command]
pub async fn ssh_write(
    state: State<'_, AppState>,
    terminal_id: String,
    data: Vec<u8>,
) -> CmdResult<()> {
    if let Some(handle) = state.terminals.get(&terminal_id) {
        handle.write_tx.send(data).await.map_err(to_str)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn ssh_resize(
    state: State<'_, AppState>,
    terminal_id: String,
    cols: u32,
    rows: u32,
) -> CmdResult<()> {
    if let Some(handle) = state.terminals.get(&terminal_id) {
        let _ = handle.resize_tx.send((cols, rows)).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn ssh_disconnect(
    state: State<'_, AppState>,
    terminal_id: String,
) -> CmdResult<()> {
    state.terminals.remove(&terminal_id);
    Ok(())
}
