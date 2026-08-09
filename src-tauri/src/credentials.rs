use anyhow::{bail, Context, Result};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use zeroize::Zeroize;

use crate::db::{CredentialMetadata, SqliteSessionStorage};
use crate::models::{AuthMethod, ConnectSecret, CredentialKind, LegacyCredentialSummary, Session};
use crate::storage::SessionStorage;

const SERVICE_NAME: &str = "com.wssh.session-credentials";

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("credential provider unavailable")]
    Unavailable,
    #[error("credential is missing")]
    Missing,
}

pub trait CredentialProvider: Send + Sync + 'static {
    fn set(&self, account: &str, secret: &str) -> Result<(), ProviderError>;
    fn get(&self, account: &str) -> Result<String, ProviderError>;
    fn delete(&self, account: &str) -> Result<(), ProviderError>;
}

pub struct KeyringCredentialProvider;

impl KeyringCredentialProvider {
    fn entry(account: &str) -> Result<keyring::Entry, ProviderError> {
        keyring::Entry::new(SERVICE_NAME, account).map_err(|_| ProviderError::Unavailable)
    }

    fn map_error(error: keyring::Error) -> ProviderError {
        match error {
            keyring::Error::NoEntry => ProviderError::Missing,
            _ => ProviderError::Unavailable,
        }
    }
}

impl CredentialProvider for KeyringCredentialProvider {
    fn set(&self, account: &str, secret: &str) -> Result<(), ProviderError> {
        Self::entry(account)?
            .set_password(secret)
            .map_err(Self::map_error)
    }

    fn get(&self, account: &str) -> Result<String, ProviderError> {
        Self::entry(account)?
            .get_password()
            .map_err(Self::map_error)
    }

    fn delete(&self, account: &str) -> Result<(), ProviderError> {
        match Self::entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(Self::map_error(error)),
        }
    }
}

pub struct CredentialService {
    provider: Arc<dyn CredentialProvider>,
    sqlite: Arc<SqliteSessionStorage>,
    gate: Mutex<()>,
}

impl CredentialService {
    pub fn new(provider: Arc<dyn CredentialProvider>, sqlite: Arc<SqliteSessionStorage>) -> Self {
        Self {
            provider,
            sqlite,
            gate: Mutex::new(()),
        }
    }

    pub async fn store(
        &self,
        session: &Session,
        kind: CredentialKind,
        secret: String,
    ) -> Result<()> {
        validate_kind(session, kind)?;
        if secret.is_empty() {
            bail!("凭据不能为空");
        }
        let _guard = self.gate.lock().await;
        let account = account_name(&session.id, kind);
        let provider = self.provider.clone();
        let verification_secret = secret.clone();
        let account_for_write = account.clone();
        tokio::task::spawn_blocking(move || {
            let mut secret = secret;
            let result = provider.set(&account_for_write, &secret);
            secret.zeroize();
            result
        })
        .await
        .context("凭据提供器写入任务失败")?
        .map_err(provider_message)?;

        let provider = self.provider.clone();
        let account_for_read = account.clone();
        let mut read_back = tokio::task::spawn_blocking(move || provider.get(&account_for_read))
            .await
            .context("凭据提供器校验任务失败")?
            .map_err(provider_message)?;
        let verified = read_back.as_bytes() == verification_secret.as_bytes();
        read_back.zeroize();
        let mut verification_secret = verification_secret;
        verification_secret.zeroize();
        if !verified {
            let provider = self.provider.clone();
            let _ = tokio::task::spawn_blocking(move || provider.delete(&account)).await;
            bail!("凭据提供器写后校验失败；未修改会话元数据");
        }

        if let Err(error) = self.sqlite.set_credential_metadata(session, kind).await {
            let provider = self.provider.clone();
            let _ = tokio::task::spawn_blocking(move || provider.delete(&account)).await;
            return Err(error).context("凭据已回滚，无法保存非敏感元数据");
        }
        Ok(())
    }

    pub async fn resolve_after_host_check(
        &self,
        session: &Session,
        one_time: Option<ConnectSecret>,
    ) -> Result<Option<String>> {
        let kind = expected_kind(session.auth_method);
        if let Some(mut supplied) = one_time {
            validate_kind(session, supplied.kind)?;
            if supplied.secret.is_empty() {
                supplied.secret.zeroize();
                bail!("一次性凭据不能为空");
            }
            return Ok(Some(supplied.secret));
        }

        let Some(metadata) = self.sqlite.credential_metadata(&session.id, kind).await? else {
            return Ok(None);
        };
        validate_binding(session, &metadata)?;

        let _guard = self.gate.lock().await;
        let provider = self.provider.clone();
        let account = account_name(&session.id, kind);
        let secret = tokio::task::spawn_blocking(move || provider.get(&account))
            .await
            .context("凭据提供器读取任务失败")?
            .map_err(provider_message)?;
        Ok(Some(secret))
    }

    pub async fn migrate_legacy(&self, session: &Session) -> Result<bool> {
        if session.auth_method != AuthMethod::Password {
            bail!("仅密码认证会话可迁移旧密码");
        }
        let Some(mut secret) = self.sqlite.legacy_password(&session.id).await? else {
            return Ok(false);
        };
        let result = self
            .store(session, CredentialKind::Password, secret.clone())
            .await;
        secret.zeroize();
        result?;
        self.sqlite
            .clear_legacy_password(&session.id)
            .await
            .context("系统凭据已校验，但旧 SQLite 密码尚未清除；可安全重试迁移")?;
        Ok(true)
    }

    pub async fn migrate_all_legacy(&self) -> Result<usize> {
        let sessions = self.sqlite.list().await?;
        let mut migrated = 0;
        for session in &sessions {
            if session.auth_method == AuthMethod::Password
                && self.sqlite.legacy_password(&session.id).await?.is_some()
                && self.migrate_legacy(session).await?
            {
                migrated += 1;
            }
        }
        Ok(migrated)
    }

    pub async fn legacy_summary(&self) -> Result<LegacyCredentialSummary> {
        let session_ids = self.sqlite.legacy_summary().await?;
        Ok(LegacyCredentialSummary {
            count: session_ids.len() as i64,
            session_ids,
        })
    }

    pub async fn delete_legacy(&self, session_id: &str) -> Result<bool> {
        self.sqlite.clear_legacy_password(session_id).await
    }

    pub async fn delete_all_legacy(&self) -> Result<usize> {
        let session_ids = self.sqlite.legacy_summary().await?;
        let mut deleted = 0;
        for session_id in session_ids {
            if self.sqlite.clear_legacy_password(&session_id).await? {
                deleted += 1;
            }
        }
        Ok(deleted)
    }

    pub async fn delete(&self, session: &Session, kind: CredentialKind) -> Result<()> {
        validate_kind(session, kind)?;
        let _guard = self.gate.lock().await;
        let provider = self.provider.clone();
        let account = account_name(&session.id, kind);
        tokio::task::spawn_blocking(move || provider.delete(&account))
            .await
            .context("凭据提供器删除任务失败")?
            .map_err(provider_message)?;
        self.sqlite
            .delete_credential_metadata(&session.id, kind)
            .await?;
        Ok(())
    }

    pub async fn delete_if_present(&self, session: &Session) -> Result<()> {
        let kind = expected_kind(session.auth_method);
        if self
            .sqlite
            .credential_metadata(&session.id, kind)
            .await?
            .is_some()
        {
            self.delete(session, kind).await?;
        }
        Ok(())
    }

    pub async fn confirm_rebind(&self, session: &Session, kind: CredentialKind) -> Result<()> {
        validate_kind(session, kind)?;
        let _guard = self.gate.lock().await;
        let provider = self.provider.clone();
        let account = account_name(&session.id, kind);
        let mut secret = tokio::task::spawn_blocking(move || provider.get(&account))
            .await
            .context("凭据提供器校验任务失败")?
            .map_err(provider_message)?;
        secret.zeroize();
        self.sqlite.set_credential_metadata(session, kind).await
    }
}

fn expected_kind(auth: AuthMethod) -> CredentialKind {
    match auth {
        AuthMethod::Password => CredentialKind::Password,
        AuthMethod::PrivateKey => CredentialKind::PrivateKeyPassphrase,
    }
}

fn validate_kind(session: &Session, kind: CredentialKind) -> Result<()> {
    if expected_kind(session.auth_method) != kind {
        bail!("凭据类型与会话认证方式不匹配");
    }
    Ok(())
}

fn validate_binding(session: &Session, metadata: &CredentialMetadata) -> Result<()> {
    if metadata.needs_rebind
        || metadata.kind != expected_kind(session.auth_method)
        || metadata.host != session.host
        || metadata.port != session.port
        || metadata.username != session.username
        || metadata.auth_method != session.auth_method
    {
        bail!("凭据目标已变化；请先确认重新绑定");
    }
    Ok(())
}

fn account_name(session_id: &str, kind: CredentialKind) -> String {
    let suffix = match kind {
        CredentialKind::Password => "password",
        CredentialKind::PrivateKeyPassphrase => "private-key-passphrase",
    };
    format!("session:{session_id}:{suffix}")
}

fn provider_message(error: ProviderError) -> anyhow::Error {
    match error {
        ProviderError::Unavailable => {
            anyhow::anyhow!("系统凭据库当前不可用；可取消保存并改用本次连接输入")
        }
        ProviderError::Missing => anyhow::anyhow!("系统凭据不存在；请重新输入"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateSession, CredentialState};
    use crate::storage::SessionStorage;
    use std::collections::HashMap;
    use std::sync::Mutex as StdMutex;
    use uuid::Uuid;

    #[derive(Default)]
    struct MemoryProvider {
        values: StdMutex<HashMap<String, String>>,
        fail_set: StdMutex<bool>,
    }

    impl CredentialProvider for MemoryProvider {
        fn set(&self, account: &str, secret: &str) -> Result<(), ProviderError> {
            if *self.fail_set.lock().unwrap() {
                return Err(ProviderError::Unavailable);
            }
            self.values
                .lock()
                .unwrap()
                .insert(account.into(), secret.into());
            Ok(())
        }
        fn get(&self, account: &str) -> Result<String, ProviderError> {
            self.values
                .lock()
                .unwrap()
                .get(account)
                .cloned()
                .ok_or(ProviderError::Missing)
        }
        fn delete(&self, account: &str) -> Result<(), ProviderError> {
            self.values.lock().unwrap().remove(account);
            Ok(())
        }
    }

    async fn fixture() -> (std::path::PathBuf, Arc<SqliteSessionStorage>, Session) {
        let path = std::env::temp_dir().join(format!("w-ssh-provider-{}.db", Uuid::new_v4()));
        let sqlite = Arc::new(
            SqliteSessionStorage::open(path.to_str().unwrap())
                .await
                .unwrap(),
        );
        let session = sqlite
            .create(CreateSession {
                name: "test".into(),
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
        (path, sqlite, session)
    }

    #[tokio::test]
    async fn stores_and_resolves_only_through_in_memory_provider() {
        let (path, sqlite, session) = fixture().await;
        let provider = Arc::new(MemoryProvider::default());
        let service = CredentialService::new(provider, sqlite.clone());
        service
            .store(&session, CredentialKind::Password, "fixture".into())
            .await
            .unwrap();
        assert_eq!(
            sqlite.credential_state(&session).await.unwrap(),
            CredentialState::Stored
        );
        assert_eq!(
            service
                .resolve_after_host_check(&session, None)
                .await
                .unwrap()
                .as_deref(),
            Some("fixture")
        );
        drop(service);
        drop(sqlite);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn provider_failure_keeps_metadata_and_sqlite_secret_free() {
        let (path, sqlite, session) = fixture().await;
        let provider = Arc::new(MemoryProvider::default());
        *provider.fail_set.lock().unwrap() = true;
        let service = CredentialService::new(provider, sqlite.clone());
        assert!(service
            .store(&session, CredentialKind::Password, "fixture".into())
            .await
            .is_err());
        assert_eq!(
            sqlite.credential_state(&session).await.unwrap(),
            CredentialState::None
        );
        assert!(sqlite.legacy_password(&session.id).await.unwrap().is_none());
        drop(service);
        drop(sqlite);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn explicit_legacy_migration_verifies_before_clearing() {
        let (path, sqlite, session) = fixture().await;
        sqlx::query("UPDATE sessions SET password='legacy-fixture' WHERE id=?")
            .bind(&session.id)
            .execute(&sqlite.pool)
            .await
            .unwrap();
        let service = CredentialService::new(Arc::new(MemoryProvider::default()), sqlite.clone());
        assert!(service.migrate_legacy(&session).await.unwrap());
        assert!(sqlite.legacy_password(&session.id).await.unwrap().is_none());
        assert_eq!(
            service
                .resolve_after_host_check(&session, None)
                .await
                .unwrap()
                .as_deref(),
            Some("legacy-fixture")
        );
        drop(service);
        drop(sqlite);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn failed_legacy_migration_keeps_plaintext_quarantined() {
        let (path, sqlite, session) = fixture().await;
        sqlx::query("UPDATE sessions SET password='legacy-fixture' WHERE id=?")
            .bind(&session.id)
            .execute(&sqlite.pool)
            .await
            .unwrap();
        let provider = Arc::new(MemoryProvider::default());
        *provider.fail_set.lock().unwrap() = true;
        let service = CredentialService::new(provider, sqlite.clone());
        assert!(service.migrate_legacy(&session).await.is_err());
        assert_eq!(
            sqlite
                .legacy_password(&session.id)
                .await
                .unwrap()
                .as_deref(),
            Some("legacy-fixture")
        );
        assert_eq!(
            sqlite.credential_state(&session).await.unwrap(),
            CredentialState::LegacyPlaintext
        );
        drop(service);
        drop(sqlite);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn target_change_requires_rebind_but_one_time_secret_still_works() {
        let (path, sqlite, session) = fixture().await;
        let service = CredentialService::new(Arc::new(MemoryProvider::default()), sqlite.clone());
        service
            .store(&session, CredentialKind::Password, "stored-fixture".into())
            .await
            .unwrap();
        let mut changed = session.clone();
        changed.host = "changed.test".into();
        sqlite.mark_credentials_for_rebind(&changed).await.unwrap();
        assert_eq!(
            sqlite.credential_state(&changed).await.unwrap(),
            CredentialState::NeedsRebind
        );
        assert!(service
            .resolve_after_host_check(&changed, None)
            .await
            .is_err());
        let one_time = service
            .resolve_after_host_check(
                &changed,
                Some(ConnectSecret {
                    kind: CredentialKind::Password,
                    secret: "one-time".into(),
                }),
            )
            .await
            .unwrap();
        assert_eq!(one_time.as_deref(), Some("one-time"));
        drop(service);
        drop(sqlite);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn verified_provider_write_does_not_hide_uncleared_legacy_value() {
        let (path, sqlite, session) = fixture().await;
        sqlx::query("UPDATE sessions SET password='legacy-fixture' WHERE id=?")
            .bind(&session.id)
            .execute(&sqlite.pool)
            .await
            .unwrap();
        let service = CredentialService::new(Arc::new(MemoryProvider::default()), sqlite.clone());
        service
            .store(&session, CredentialKind::Password, "legacy-fixture".into())
            .await
            .unwrap();
        assert_eq!(
            sqlite.credential_state(&session).await.unwrap(),
            CredentialState::LegacyPlaintext
        );
        assert!(service.migrate_legacy(&session).await.unwrap());
        assert_eq!(
            sqlite.credential_state(&session).await.unwrap(),
            CredentialState::Stored
        );
        drop(service);
        drop(sqlite);
        let _ = std::fs::remove_file(path);
    }
}
