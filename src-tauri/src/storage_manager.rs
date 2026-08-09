use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::SqliteSessionStorage;
use crate::models::{CreateSession, Session, UpdateGroup, UpdateSession};
use crate::storage::SessionStorage;
use crate::yaml_storage::YamlSessionStorage;

const SETTINGS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackend {
    Sqlite,
    Yaml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSelection {
    pub backend: StorageBackend,
    pub yaml_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageStatus {
    pub backend: StorageBackend,
    pub yaml_path: Option<String>,
    pub sqlite_path: String,
    pub active_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageCopyResult {
    pub copied: usize,
    pub status: StorageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StorageSettingsFile {
    schema_version: u32,
    backend: StorageBackend,
    yaml_path: Option<String>,
}

impl Default for StorageSettingsFile {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            backend: StorageBackend::Sqlite,
            yaml_path: None,
        }
    }
}

struct ActiveStorage {
    settings: StorageSettingsFile,
    storage: Option<Arc<dyn SessionStorage>>,
    active_error: Option<String>,
}

pub struct StorageManager {
    sqlite_path: PathBuf,
    settings_path: PathBuf,
    sqlite: Arc<SqliteSessionStorage>,
    active: RwLock<ActiveStorage>,
}

impl StorageManager {
    pub async fn open(app_data_dir: PathBuf) -> Result<Self> {
        let sqlite_path = app_data_dir.join("sessions.db");
        let settings_path = app_data_dir.join("storage-settings.json");
        let sqlite = Arc::new(
            SqliteSessionStorage::open(path_str(&sqlite_path)?)
                .await
                .context("SQLite 初始化失败")?,
        );

        let (settings, settings_error) = match load_settings(&settings_path) {
            Ok(Some(settings)) => (settings, None),
            Ok(None) => (StorageSettingsFile::default(), None),
            Err(error) => (
                StorageSettingsFile::default(),
                Some(format!(
                    "存储设置损坏，已保持 SQLite 默认后端；请重新应用设置：{error}"
                )),
            ),
        };

        let (storage, active_error) = if settings_error.is_some() {
            (
                Some(sqlite.clone() as Arc<dyn SessionStorage>),
                settings_error,
            )
        } else {
            match open_selected_storage(&settings, &sqlite).await {
                Ok(storage) => (Some(storage), None),
                Err(error) => (
                    None,
                    Some(format!("已选择的存储不可用，未静默回退：{error}")),
                ),
            }
        };

        Ok(Self {
            sqlite_path,
            settings_path,
            sqlite,
            active: RwLock::new(ActiveStorage {
                settings,
                storage,
                active_error,
            }),
        })
    }

    pub async fn status(&self) -> StorageStatus {
        let active = self.active.read().await;
        self.status_from(&active)
    }

    pub async fn apply(&self, selection: StorageSelection) -> Result<StorageStatus> {
        let mut active = self.active.write().await;
        let settings = normalize_selection(selection)?;
        let storage = open_selected_storage(&settings, &self.sqlite).await?;
        persist_settings(&self.settings_path, &settings)?;
        active.settings = settings;
        active.storage = Some(storage);
        active.active_error = None;
        Ok(self.status_from(&active))
    }

    pub async fn copy_and_switch(&self, selection: StorageSelection) -> Result<StorageCopyResult> {
        let mut active = self.active.write().await;
        let source = active
            .storage
            .clone()
            .ok_or_else(|| anyhow!(active_error(&active)))?;
        let target_settings = normalize_selection(selection)?;
        if target_settings.backend == active.settings.backend {
            bail!("复制目标必须是另一个存储后端");
        }

        let target = open_selected_storage(&target_settings, &self.sqlite).await?;
        if !target.list().await?.is_empty() {
            bail!("目标存储非空，复制已取消；源数据和当前后端未改变");
        }

        let source_sessions = source.list().await?;
        target.import_if_empty(source_sessions.clone()).await?;
        let copied_sessions = target.list().await?;
        verify_copy(
            source_sessions.clone(),
            copied_sessions,
            target_settings.backend,
        )?;

        persist_settings(&self.settings_path, &target_settings)?;
        active.settings = target_settings;
        active.storage = Some(target);
        active.active_error = None;

        Ok(StorageCopyResult {
            copied: source_sessions.len(),
            status: self.status_from(&active),
        })
    }

    fn status_from(&self, active: &ActiveStorage) -> StorageStatus {
        StorageStatus {
            backend: active.settings.backend,
            yaml_path: active.settings.yaml_path.clone(),
            sqlite_path: self.sqlite_path.to_string_lossy().into_owned(),
            active_error: active.active_error.clone(),
        }
    }

    async fn current_storage(&self) -> Result<Arc<dyn SessionStorage>> {
        let active = self.active.read().await;
        active
            .storage
            .clone()
            .ok_or_else(|| anyhow!(active_error(&active)))
    }

    pub fn sqlite(&self) -> Arc<SqliteSessionStorage> {
        self.sqlite.clone()
    }

    async fn decorate(&self, mut session: Session) -> Result<Session> {
        session.credential_state = self.sqlite.credential_state(&session).await?;
        Ok(session)
    }
}

#[async_trait]
impl SessionStorage for StorageManager {
    async fn list(&self) -> Result<Vec<Session>> {
        let sessions = self.current_storage().await?.list().await?;
        let mut decorated = Vec::with_capacity(sessions.len());
        for session in sessions {
            decorated.push(self.decorate(session).await?);
        }
        Ok(decorated)
    }

    async fn get(&self, id: &str) -> Result<Session> {
        let session = self.current_storage().await?.get(id).await?;
        self.decorate(session).await
    }

    async fn create(&self, data: CreateSession) -> Result<Session> {
        let session = self.current_storage().await?.create(data).await?;
        self.decorate(session).await
    }

    async fn update(&self, data: UpdateSession) -> Result<Session> {
        let session = self.current_storage().await?.update(data).await?;
        self.sqlite.mark_credentials_for_rebind(&session).await?;
        self.decorate(session).await
    }

    async fn update_group(&self, data: UpdateGroup) -> Result<Vec<Session>> {
        let sessions = self.current_storage().await?.update_group(data).await?;
        let mut decorated = Vec::with_capacity(sessions.len());
        for session in sessions {
            decorated.push(self.decorate(session).await?);
        }
        Ok(decorated)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        self.current_storage().await?.delete(id).await
    }

    async fn import_if_empty(&self, sessions: Vec<Session>) -> Result<()> {
        self.current_storage()
            .await?
            .import_if_empty(sessions)
            .await
    }

    async fn take_notice(&self) -> Option<String> {
        match self.current_storage().await {
            Ok(storage) => storage.take_notice().await,
            Err(_) => None,
        }
    }
}

fn normalize_selection(selection: StorageSelection) -> Result<StorageSettingsFile> {
    let yaml_path = selection
        .yaml_path
        .map(|path| path.trim().to_owned())
        .filter(|path| !path.is_empty());
    if selection.backend == StorageBackend::Yaml {
        let path = yaml_path
            .as_deref()
            .ok_or_else(|| anyhow!("选择 YAML 后端时必须提供绝对 .yml 路径"))?;
        if !Path::new(path).is_absolute() {
            bail!("YAML 配置路径必须是绝对路径");
        }
    }
    Ok(StorageSettingsFile {
        schema_version: SETTINGS_SCHEMA_VERSION,
        backend: selection.backend,
        yaml_path,
    })
}

async fn open_selected_storage(
    settings: &StorageSettingsFile,
    sqlite: &Arc<SqliteSessionStorage>,
) -> Result<Arc<dyn SessionStorage>> {
    match settings.backend {
        StorageBackend::Sqlite => Ok(sqlite.clone()),
        StorageBackend::Yaml => {
            let path = settings
                .yaml_path
                .as_deref()
                .ok_or_else(|| anyhow!("YAML 后端缺少配置路径"))?;
            Ok(Arc::new(
                YamlSessionStorage::open(PathBuf::from(path)).await?,
            ))
        }
    }
}

fn load_settings(path: &Path) -> Result<Option<StorageSettingsFile>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw =
        std::fs::read(path).with_context(|| format!("无法读取存储设置: {}", path.display()))?;
    let settings: StorageSettingsFile =
        serde_json::from_slice(&raw).context("存储设置 JSON 无效")?;
    if settings.schema_version != SETTINGS_SCHEMA_VERSION {
        bail!(
            "不支持的存储设置 schema_version: {}",
            settings.schema_version
        );
    }
    normalize_selection(StorageSelection {
        backend: settings.backend,
        yaml_path: settings.yaml_path,
    })
    .map(Some)
}

fn persist_settings(path: &Path, settings: &StorageSettingsFile) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(settings)?;
    bytes.push(b'\n');
    let mut file = AtomicWriteFile::open(path)
        .with_context(|| format!("无法创建存储设置临时文件: {}", path.display()))?;
    file.write_all(&bytes)?;
    file.flush()?;
    file.sync_all()?;
    file.commit()
        .with_context(|| format!("无法原子更新存储设置: {}", path.display()))?;
    Ok(())
}

fn verify_copy(
    mut source: Vec<Session>,
    mut target: Vec<Session>,
    _backend: StorageBackend,
) -> Result<()> {
    for session in &mut source {
        session.credential_state = Default::default();
    }
    for session in &mut target {
        session.credential_state = Default::default();
    }
    source.sort_by(|a, b| a.id.cmp(&b.id));
    target.sort_by(|a, b| a.id.cmp(&b.id));
    if source != target {
        bail!("复制后逐字段校验失败；当前后端未切换，源数据未删除");
    }
    Ok(())
}

fn active_error(active: &ActiveStorage) -> String {
    active
        .active_error
        .clone()
        .unwrap_or_else(|| "当前存储不可用".into())
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str().ok_or_else(|| anyhow!("路径不是有效 UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AuthMethod;
    use uuid::Uuid;

    fn temp_root() -> PathBuf {
        let path = std::env::temp_dir().join(format!("w-ssh-manager-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn fixture(name: &str) -> CreateSession {
        CreateSession {
            name: name.into(),
            host: format!("{name}.test"),
            port: 22,
            username: "tester".into(),
            private_key: None,
            auth_method: AuthMethod::Password,
            icon: Some("server".into()),
            group_name: Some("fixtures".into()),
            group_icon: Some("folder".into()),
        }
    }

    #[tokio::test]
    async fn defaults_to_existing_sqlite_path_without_creating_settings() {
        let root = temp_root();
        let manager = StorageManager::open(root.clone()).await.unwrap();
        assert_eq!(manager.status().await.backend, StorageBackend::Sqlite);
        assert!(root.join("sessions.db").exists());
        assert!(!root.join("storage-settings.json").exists());
        manager.create(fixture("default")).await.unwrap();
        assert_eq!(manager.list().await.unwrap().len(), 1);
        drop(manager);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn copy_to_yaml_never_exports_quarantined_legacy_password() {
        let root = temp_root();
        let yaml_path = root.join("sessions.yml");
        let manager = StorageManager::open(root.clone()).await.unwrap();
        let created = manager.create(fixture("copy")).await.unwrap();
        sqlx::query("UPDATE sessions SET password='fixture-password' WHERE id=?")
            .bind(&created.id)
            .execute(&manager.sqlite.pool)
            .await
            .unwrap();
        let result = manager
            .copy_and_switch(StorageSelection {
                backend: StorageBackend::Yaml,
                yaml_path: Some(yaml_path.to_string_lossy().into_owned()),
            })
            .await
            .unwrap();
        assert_eq!(result.copied, 1);
        assert_eq!(result.status.backend, StorageBackend::Yaml);
        let copied = manager.get(&created.id).await.unwrap();
        assert_eq!(
            copied.credential_state,
            crate::models::CredentialState::LegacyPlaintext
        );
        assert!(!std::fs::read_to_string(&yaml_path)
            .unwrap()
            .contains("fixture-password"));

        let source = SqliteSessionStorage::open(path_str(&root.join("sessions.db")).unwrap())
            .await
            .unwrap();
        assert_eq!(
            source
                .legacy_password(&created.id)
                .await
                .unwrap()
                .as_deref(),
            Some("fixture-password")
        );
        drop(manager);
        drop(source);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn refuses_nonempty_copy_target_without_switching() {
        let root = temp_root();
        let yaml_path = root.join("occupied.yml");
        let manager = StorageManager::open(root.clone()).await.unwrap();
        manager.create(fixture("source")).await.unwrap();
        let occupied = YamlSessionStorage::open(yaml_path.clone()).await.unwrap();
        occupied.create(fixture("target")).await.unwrap();

        let error = manager
            .copy_and_switch(StorageSelection {
                backend: StorageBackend::Yaml,
                yaml_path: Some(yaml_path.to_string_lossy().into_owned()),
            })
            .await
            .unwrap_err();
        assert!(error.to_string().contains("非空"));
        assert_eq!(manager.status().await.backend, StorageBackend::Sqlite);
        assert_eq!(manager.list().await.unwrap().len(), 1);
        drop(manager);
        drop(occupied);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn corrupted_selected_yaml_is_reported_without_silent_fallback() {
        let root = temp_root();
        let yaml_path = root.join("selected.yml");
        let manager = StorageManager::open(root.clone()).await.unwrap();
        manager
            .apply(StorageSelection {
                backend: StorageBackend::Yaml,
                yaml_path: Some(yaml_path.to_string_lossy().into_owned()),
            })
            .await
            .unwrap();
        drop(manager);
        std::fs::write(&yaml_path, "schema_version: 1\nsessions: [\n").unwrap();

        let reopened = StorageManager::open(root.clone()).await.unwrap();
        let status = reopened.status().await;
        assert_eq!(status.backend, StorageBackend::Yaml);
        assert!(status.active_error.unwrap().contains("未静默回退"));
        assert!(reopened.list().await.is_err());
        reopened
            .apply(StorageSelection {
                backend: StorageBackend::Sqlite,
                yaml_path: Some(yaml_path.to_string_lossy().into_owned()),
            })
            .await
            .unwrap();
        assert_eq!(reopened.status().await.backend, StorageBackend::Sqlite);
        drop(reopened);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn copy_from_yaml_to_empty_sqlite_keeps_yaml_source() {
        let root = temp_root();
        let yaml_path = root.join("source.yml");
        let yaml = YamlSessionStorage::open(yaml_path.clone()).await.unwrap();
        let created = yaml.create(fixture("yaml-source")).await.unwrap();
        drop(yaml);

        let manager = StorageManager::open(root.clone()).await.unwrap();
        manager
            .apply(StorageSelection {
                backend: StorageBackend::Yaml,
                yaml_path: Some(yaml_path.to_string_lossy().into_owned()),
            })
            .await
            .unwrap();
        let result = manager
            .copy_and_switch(StorageSelection {
                backend: StorageBackend::Sqlite,
                yaml_path: Some(yaml_path.to_string_lossy().into_owned()),
            })
            .await
            .unwrap();

        assert_eq!(result.copied, 1);
        assert_eq!(result.status.backend, StorageBackend::Sqlite);
        assert_eq!(manager.get(&created.id).await.unwrap(), created);
        let source = YamlSessionStorage::open(yaml_path.clone()).await.unwrap();
        assert_eq!(source.get(&created.id).await.unwrap(), created);
        drop(manager);
        drop(source);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn settings_commit_failure_does_not_switch_or_remove_source() {
        let root = temp_root();
        let yaml_path = root.join("verified-target.yml");
        let manager = StorageManager::open(root.clone()).await.unwrap();
        let created = manager.create(fixture("rollback")).await.unwrap();
        std::fs::create_dir(root.join("storage-settings.json")).unwrap();

        let error = manager
            .copy_and_switch(StorageSelection {
                backend: StorageBackend::Yaml,
                yaml_path: Some(yaml_path.to_string_lossy().into_owned()),
            })
            .await
            .unwrap_err();
        assert!(error.to_string().contains("存储设置"));
        assert_eq!(manager.status().await.backend, StorageBackend::Sqlite);
        assert_eq!(manager.get(&created.id).await.unwrap(), created);

        let verified_target = YamlSessionStorage::open(yaml_path).await.unwrap();
        let target_session = verified_target.get(&created.id).await.unwrap();
        assert_eq!(
            target_session.credential_state,
            crate::models::CredentialState::None
        );
        drop(manager);
        drop(verified_target);
        let _ = std::fs::remove_dir_all(root);
    }
}
