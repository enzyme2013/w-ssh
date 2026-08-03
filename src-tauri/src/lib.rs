mod commands;
mod db;
mod models;
mod ssh;
mod storage;
mod storage_manager;
mod yaml_storage;

use ssh::TerminalMap;
use std::sync::Arc;
use tauri::Manager;

pub struct AppState {
    pub storage: Arc<storage_manager::StorageManager>,
    pub terminals: TerminalMap,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("无法获取数据目录");
            std::fs::create_dir_all(&data_dir).expect("无法创建数据目录");
            let storage =
                tauri::async_runtime::block_on(storage_manager::StorageManager::open(data_dir))
                    .expect("存储初始化失败");

            let terminals: TerminalMap = Arc::new(dashmap::DashMap::new());

            app.manage(AppState {
                storage: Arc::new(storage),
                terminals,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_sessions,
            commands::create_session,
            commands::update_session,
            commands::delete_session,
            commands::get_storage_status,
            commands::set_storage_settings,
            commands::copy_storage_and_switch,
            commands::take_storage_notice,
            commands::get_ssh_key_paths,
            commands::ssh_connect,
            commands::ssh_write,
            commands::ssh_resize,
            commands::ssh_disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
