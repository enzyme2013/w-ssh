mod commands;
mod db;
mod models;
mod ssh;

use ssh::TerminalMap;
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::Manager;

pub struct AppState {
    pub db: SqlitePool,
    pub terminals: TerminalMap,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("无法获取数据目录");
            std::fs::create_dir_all(&data_dir).expect("无法创建数据目录");
            let db_path = data_dir.join("sessions.db");
            let db_path_str = db_path.to_str().unwrap().to_string();

            let db = tauri::async_runtime::block_on(db::init_db(&db_path_str))
                .expect("数据库初始化失败");

            let terminals: TerminalMap = Arc::new(dashmap::DashMap::new());

            app.manage(AppState { db, terminals });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_sessions,
            commands::create_session,
            commands::update_session,
            commands::delete_session,
            commands::get_ssh_key_paths,
            commands::ssh_connect,
            commands::ssh_write,
            commands::ssh_resize,
            commands::ssh_disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
