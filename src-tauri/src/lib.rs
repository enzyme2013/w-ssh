mod commands;
mod credentials;
mod db;
mod host_trust;
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
    pub credentials: Arc<credentials::CredentialService>,
    pub host_trust: Arc<host_trust::HostTrustStore>,
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
            let sqlite = storage.sqlite();
            let credentials = Arc::new(credentials::CredentialService::new(
                Arc::new(credentials::KeyringCredentialProvider),
                sqlite,
            ));
            let host_trust = Arc::new(host_trust::HostTrustStore::new(
                app.path()
                    .app_data_dir()
                    .expect("无法获取数据目录")
                    .join("known_hosts"),
            ));

            let terminals: TerminalMap = Arc::new(dashmap::DashMap::new());

            app.manage(AppState {
                storage: Arc::new(storage),
                credentials,
                host_trust,
                terminals,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_sessions,
            commands::create_session,
            commands::update_session,
            commands::update_group,
            commands::delete_session,
            commands::get_storage_status,
            commands::set_storage_settings,
            commands::copy_storage_and_switch,
            commands::take_storage_notice,
            commands::get_ssh_key_paths,
            commands::set_session_credential,
            commands::delete_session_credential,
            commands::confirm_session_credential_rebind,
            commands::get_legacy_credential_summary,
            commands::migrate_legacy_credential,
            commands::migrate_all_legacy_credentials,
            commands::delete_legacy_credential,
            commands::delete_all_legacy_credentials,
            commands::ssh_trust_preflight,
            commands::accept_host_trust,
            commands::replace_host_trust,
            commands::get_host_trust_entries,
            commands::delete_host_trust,
            commands::ssh_connect,
            commands::ssh_write,
            commands::ssh_resize,
            commands::ssh_disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
