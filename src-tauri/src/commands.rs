use crate::{
    models::*,
    ssh,
    storage::SessionStorage,
    storage_manager::{StorageCopyResult, StorageSelection, StorageStatus},
    AppState,
};
use tauri::{AppHandle, State};

type CmdResult<T> = Result<T, String>;

fn to_str<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// ── 会话管理 ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_sessions(state: State<'_, AppState>) -> CmdResult<Vec<Session>> {
    state.storage.list().await.map_err(to_str)
}

#[tauri::command]
pub async fn create_session(state: State<'_, AppState>, data: CreateSession) -> CmdResult<Session> {
    state.storage.create(data).await.map_err(to_str)
}

#[tauri::command]
pub async fn update_session(state: State<'_, AppState>, data: UpdateSession) -> CmdResult<Session> {
    state.storage.update(data).await.map_err(to_str)
}

#[tauri::command]
pub async fn delete_session(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.storage.delete(&id).await.map_err(to_str)
}

#[tauri::command]
pub async fn get_storage_status(state: State<'_, AppState>) -> CmdResult<StorageStatus> {
    Ok(state.storage.status().await)
}

#[tauri::command]
pub async fn set_storage_settings(
    state: State<'_, AppState>,
    selection: StorageSelection,
) -> CmdResult<StorageStatus> {
    state.storage.apply(selection).await.map_err(to_str)
}

#[tauri::command]
pub async fn copy_storage_and_switch(
    state: State<'_, AppState>,
    selection: StorageSelection,
) -> CmdResult<StorageCopyResult> {
    state
        .storage
        .copy_and_switch(selection)
        .await
        .map_err(to_str)
}

#[tauri::command]
pub async fn take_storage_notice(state: State<'_, AppState>) -> CmdResult<Option<String>> {
    Ok(state.storage.take_notice().await)
}

// ── 工具 ────────────────────────────────────────────────────────────────────

/// 扫描 ~/.ssh/ 下的常见私钥文件，返回存在的绝对路径列表
#[tauri::command]
pub fn get_ssh_key_paths() -> Vec<String> {
    let common_names = [
        "id_rsa",
        "id_ed25519",
        "id_ecdsa",
        "id_dsa",
        "id_ecdsa_sk",
        "id_ed25519_sk",
    ];

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
    password: Option<String>,
) -> CmdResult<String> {
    let session = state.storage.get(&session_id).await.map_err(to_str)?;

    let terminal_id = uuid::Uuid::new_v4().to_string();

    ssh::connect(
        app,
        terminal_id.clone(),
        state.terminals.clone(),
        session.host,
        session.port as u16,
        session.username,
        password.or(session.password),
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
pub async fn ssh_disconnect(state: State<'_, AppState>, terminal_id: String) -> CmdResult<()> {
    state.terminals.remove(&terminal_id);
    Ok(())
}
