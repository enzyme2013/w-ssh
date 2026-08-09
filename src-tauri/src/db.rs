use anyhow::{bail, Result};
use async_trait::async_trait;
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

use crate::models::{
    AuthMethod, CreateSession, CredentialKind, CredentialState, Session, UpdateGroup, UpdateSession,
};
use crate::storage::{now_str, SessionStorage};

pub struct SqliteSessionStorage {
    pub(crate) pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct CredentialMetadata {
    pub kind: CredentialKind,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub auth_method: AuthMethod,
    pub needs_rebind: bool,
}

impl SqliteSessionStorage {
    pub async fn open(db_path: &str) -> Result<Self> {
        let opts =
            SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))?.create_if_missing(true);
        let pool = SqlitePool::connect_with(opts).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 22,
                username TEXT NOT NULL,
                password TEXT,
                private_key TEXT,
                icon TEXT,
                group_name TEXT,
                group_icon TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        for (name, definition) in [("icon", "TEXT"), ("group_icon", "TEXT")] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name=?",
            )
            .bind(name)
            .fetch_one(&pool)
            .await?;
            if exists == 0 {
                sqlx::query(&format!(
                    "ALTER TABLE sessions ADD COLUMN {name} {definition}"
                ))
                .execute(&pool)
                .await?;
            }
        }

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS credential_metadata (
                session_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                username TEXT NOT NULL,
                auth_method TEXT NOT NULL,
                needs_rebind INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (session_id, kind)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn credential_metadata(
        &self,
        session_id: &str,
        kind: CredentialKind,
    ) -> Result<Option<CredentialMetadata>> {
        let row = sqlx::query(
            "SELECT kind, host, port, username, auth_method, needs_rebind \
             FROM credential_metadata WHERE session_id=? AND kind=?",
        )
        .bind(session_id)
        .bind(kind_name(kind))
        .fetch_optional(&self.pool)
        .await?;
        row.map(|row| {
            Ok(CredentialMetadata {
                kind: parse_kind(row.get::<String, _>("kind").as_str())?,
                host: row.get("host"),
                port: row.get("port"),
                username: row.get("username"),
                auth_method: parse_auth(row.get::<String, _>("auth_method").as_str())?,
                needs_rebind: row.get::<i64, _>("needs_rebind") != 0,
            })
        })
        .transpose()
    }

    pub async fn set_credential_metadata(
        &self,
        session: &Session,
        kind: CredentialKind,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO credential_metadata \
             (session_id, kind, host, port, username, auth_method, needs_rebind, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, 0, ?) \
             ON CONFLICT(session_id, kind) DO UPDATE SET \
             host=excluded.host, port=excluded.port, username=excluded.username, \
             auth_method=excluded.auth_method, needs_rebind=0, updated_at=excluded.updated_at",
        )
        .bind(&session.id)
        .bind(kind_name(kind))
        .bind(&session.host)
        .bind(session.port)
        .bind(&session.username)
        .bind(auth_name(session.auth_method))
        .bind(now_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_credential_metadata(
        &self,
        session_id: &str,
        kind: CredentialKind,
    ) -> Result<()> {
        sqlx::query("DELETE FROM credential_metadata WHERE session_id=? AND kind=?")
            .bind(session_id)
            .bind(kind_name(kind))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn mark_credentials_for_rebind(&self, session: &Session) -> Result<()> {
        sqlx::query(
            "UPDATE credential_metadata SET needs_rebind=1, updated_at=? \
             WHERE session_id=? AND (host<>? OR port<>? OR username<>? OR auth_method<>?)",
        )
        .bind(now_str())
        .bind(&session.id)
        .bind(&session.host)
        .bind(session.port)
        .bind(&session.username)
        .bind(auth_name(session.auth_method))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn credential_state(&self, session: &Session) -> Result<CredentialState> {
        let kind = expected_kind(session.auth_method);
        if session.auth_method == AuthMethod::Password
            && self.has_legacy_password(&session.id).await?
        {
            return Ok(CredentialState::LegacyPlaintext);
        }
        if let Some(metadata) = self.credential_metadata(&session.id, kind).await? {
            return Ok(if metadata.needs_rebind {
                CredentialState::NeedsRebind
            } else {
                CredentialState::Stored
            });
        }
        Ok(CredentialState::None)
    }

    pub async fn legacy_summary(&self) -> Result<Vec<String>> {
        Ok(sqlx::query_scalar::<_, String>(
            "SELECT id FROM sessions WHERE password IS NOT NULL AND password <> '' ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn legacy_password(&self, session_id: &str) -> Result<Option<String>> {
        Ok(
            sqlx::query_scalar::<_, Option<String>>("SELECT password FROM sessions WHERE id=?")
                .bind(session_id)
                .fetch_optional(&self.pool)
                .await?
                .flatten()
                .filter(|value| !value.is_empty()),
        )
    }

    pub async fn clear_legacy_password(&self, session_id: &str) -> Result<bool> {
        let result =
            sqlx::query("UPDATE sessions SET password=NULL WHERE id=? AND password IS NOT NULL")
                .bind(session_id)
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_legacy_password(&self, session_id: &str) -> Result<bool> {
        Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sessions WHERE id=? AND password IS NOT NULL AND password <> ''",
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await?
            > 0)
    }
}

fn expected_kind(auth: AuthMethod) -> CredentialKind {
    match auth {
        AuthMethod::Password => CredentialKind::Password,
        AuthMethod::PrivateKey => CredentialKind::PrivateKeyPassphrase,
    }
}

fn kind_name(kind: CredentialKind) -> &'static str {
    match kind {
        CredentialKind::Password => "password",
        CredentialKind::PrivateKeyPassphrase => "private_key_passphrase",
    }
}

fn parse_kind(value: &str) -> Result<CredentialKind> {
    match value {
        "password" => Ok(CredentialKind::Password),
        "private_key_passphrase" => Ok(CredentialKind::PrivateKeyPassphrase),
        _ => bail!("凭据元数据类型无效"),
    }
}

fn auth_name(auth: AuthMethod) -> &'static str {
    match auth {
        AuthMethod::Password => "password",
        AuthMethod::PrivateKey => "private_key",
    }
}

fn parse_auth(value: &str) -> Result<AuthMethod> {
    match value {
        "password" => Ok(AuthMethod::Password),
        "private_key" => Ok(AuthMethod::PrivateKey),
        _ => bail!("凭据元数据认证方式无效"),
    }
}

fn row_to_session(row: &sqlx::sqlite::SqliteRow) -> Session {
    let private_key: Option<String> = row.get("private_key");
    Session {
        id: row.get("id"),
        name: row.get("name"),
        host: row.get("host"),
        port: row.get("port"),
        username: row.get("username"),
        auth_method: if private_key.is_some() {
            AuthMethod::PrivateKey
        } else {
            AuthMethod::Password
        },
        credential_state: CredentialState::None,
        private_key,
        icon: row.get("icon"),
        group_name: row.get("group_name"),
        group_icon: row.get("group_icon"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

const SESSION_COLUMNS: &str =
    "id, name, host, port, username, private_key, icon, group_name, group_icon, created_at, updated_at";

#[async_trait]
impl SessionStorage for SqliteSessionStorage {
    async fn list(&self) -> Result<Vec<Session>> {
        let sql = format!("SELECT {SESSION_COLUMNS} FROM sessions ORDER BY group_name, name");
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_session).collect())
    }

    async fn get(&self, id: &str) -> Result<Session> {
        let sql = format!("SELECT {SESSION_COLUMNS} FROM sessions WHERE id=?");
        let row = sqlx::query(&sql).bind(id).fetch_one(&self.pool).await?;
        Ok(row_to_session(&row))
    }

    async fn create(&self, data: CreateSession) -> Result<Session> {
        validate_auth(data.auth_method, data.private_key.as_deref())?;
        let id = Uuid::new_v4().to_string();
        let now = now_str();
        sqlx::query(
            "INSERT INTO sessions \
             (id, name, host, port, username, password, private_key, icon, group_name, group_icon, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&data.name)
        .bind(&data.host)
        .bind(data.port)
        .bind(&data.username)
        .bind(&data.private_key)
        .bind(&data.icon)
        .bind(&data.group_name)
        .bind(&data.group_icon)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        self.get(&id).await
    }

    async fn update(&self, data: UpdateSession) -> Result<Session> {
        validate_auth(data.auth_method, data.private_key.as_deref())?;
        let now = now_str();
        sqlx::query(
            "UPDATE sessions SET name=?, host=?, port=?, username=?, private_key=?, icon=?, group_name=?, group_icon=?, updated_at=? WHERE id=?",
        )
        .bind(&data.name)
        .bind(&data.host)
        .bind(data.port)
        .bind(&data.username)
        .bind(&data.private_key)
        .bind(&data.icon)
        .bind(&data.group_name)
        .bind(&data.group_icon)
        .bind(&now)
        .bind(&data.id)
        .execute(&self.pool)
        .await?;
        self.get(&data.id).await
    }

    async fn update_group(&self, data: UpdateGroup) -> Result<Vec<Session>> {
        let current_name = data.current_name.trim();
        let name = data.name.trim();
        validate_group(current_name, name, data.icon.as_deref())?;

        let mut tx = self.pool.begin().await?;
        if current_name != name {
            let conflict: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE group_name=?")
                    .bind(name)
                    .fetch_one(&mut *tx)
                    .await?;
            if conflict > 0 {
                bail!("目标分组已存在，请使用其他名称");
            }
        }

        let result = sqlx::query(
            "UPDATE sessions SET group_name=?, group_icon=?, updated_at=? WHERE group_name=?",
        )
        .bind(name)
        .bind(&data.icon)
        .bind(now_str())
        .bind(current_name)
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            bail!("分组不存在: {current_name}");
        }
        tx.commit().await?;

        let sql =
            format!("SELECT {SESSION_COLUMNS} FROM sessions WHERE group_name=? ORDER BY name");
        let rows = sqlx::query(&sql).bind(name).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(row_to_session).collect())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM credential_metadata WHERE session_id=?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM sessions WHERE id=?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn import_if_empty(&self, sessions: Vec<Session>) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
            .fetch_one(&mut *tx)
            .await?;
        if count != 0 {
            bail!("SQLite 目标存储非空，复制已取消");
        }
        for session in sessions {
            sqlx::query(
                "INSERT INTO sessions \
                 (id, name, host, port, username, password, private_key, icon, group_name, group_icon, created_at, updated_at) \
                 VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, ?)",
            )
            .bind(session.id)
            .bind(session.name)
            .bind(session.host)
            .bind(session.port)
            .bind(session.username)
            .bind(session.private_key)
            .bind(session.icon)
            .bind(session.group_name)
            .bind(session.group_icon)
            .bind(session.created_at)
            .bind(session.updated_at)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

fn validate_auth(auth: AuthMethod, private_key: Option<&str>) -> Result<()> {
    match (auth, private_key.filter(|value| !value.trim().is_empty())) {
        (AuthMethod::Password, Some(_)) => bail!("密码认证不能包含私钥路径"),
        (AuthMethod::PrivateKey, None) => bail!("私钥认证必须提供私钥路径"),
        _ => Ok(()),
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

    #[tokio::test]
    async fn sqlite_implements_session_storage_contract() {
        let path = std::env::temp_dir().join(format!("w-ssh-contract-{}.db", Uuid::new_v4()));
        let storage = SqliteSessionStorage::open(path.to_str().unwrap())
            .await
            .unwrap();
        assert_storage_contract(&storage).await;
        storage.pool.close().await;
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn existing_database_adds_optional_icon_columns() {
        let path = std::env::temp_dir().join(format!("w-ssh-icon-migration-{}.db", Uuid::new_v4()));
        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.display()))
            .unwrap()
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await.unwrap();
        sqlx::query(
            r#"
            CREATE TABLE sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 22,
                username TEXT NOT NULL,
                password TEXT,
                private_key TEXT,
                group_name TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;

        let storage = SqliteSessionStorage::open(path.to_str().unwrap())
            .await
            .unwrap();
        let columns: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM pragma_table_info('sessions') \
             WHERE name IN ('icon', 'group_icon') ORDER BY name",
        )
        .fetch_all(&storage.pool)
        .await
        .unwrap();
        assert_eq!(columns, vec!["group_icon", "icon"]);
        storage.pool.close().await;
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn legacy_password_is_quarantined_from_session_dto() {
        let path = std::env::temp_dir().join(format!("w-ssh-legacy-{}.db", Uuid::new_v4()));
        let storage = SqliteSessionStorage::open(path.to_str().unwrap())
            .await
            .unwrap();
        let created = storage
            .create(CreateSession {
                name: "legacy".into(),
                host: "legacy.test".into(),
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
        sqlx::query("UPDATE sessions SET password='fixture-secret' WHERE id=?")
            .bind(&created.id)
            .execute(&storage.pool)
            .await
            .unwrap();
        let json = serde_json::to_string(&storage.get(&created.id).await.unwrap()).unwrap();
        assert!(!json.contains("fixture-secret"));
        assert!(!json.contains("\"password\":"));
        assert_eq!(
            storage
                .legacy_password(&created.id)
                .await
                .unwrap()
                .as_deref(),
            Some("fixture-secret")
        );
        storage.pool.close().await;
        std::fs::remove_file(path).unwrap();
    }
}
