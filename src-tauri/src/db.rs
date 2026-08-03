use anyhow::Result;
use async_trait::async_trait;
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

use crate::models::{CreateSession, Session, UpdateSession};
use crate::storage::{now_str, SessionStorage};

pub struct SqliteSessionStorage {
    pool: SqlitePool,
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
            group_name TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
    "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

fn row_to_session(row: &sqlx::sqlite::SqliteRow) -> Session {
    Session {
        id: row.get("id"),
        name: row.get("name"),
        host: row.get("host"),
        port: row.get("port"),
        username: row.get("username"),
        password: row.get("password"),
        private_key: row.get("private_key"),
        group_name: row.get("group_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl SessionStorage for SqliteSessionStorage {
    async fn list(&self) -> Result<Vec<Session>> {
        let rows = sqlx::query(
        "SELECT id, name, host, port, username, password, private_key, group_name, created_at, updated_at \
         FROM sessions ORDER BY group_name, name"
    )
    .fetch_all(&self.pool)
    .await?;

        Ok(rows.iter().map(row_to_session).collect())
    }

    async fn get(&self, id: &str) -> Result<Session> {
        let row = sqlx::query(
        "SELECT id, name, host, port, username, password, private_key, group_name, created_at, updated_at \
         FROM sessions WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&self.pool)
    .await?;

        Ok(row_to_session(&row))
    }

    async fn create(&self, data: CreateSession) -> Result<Session> {
        let id = Uuid::new_v4().to_string();
        let now = now_str();

        sqlx::query(
        "INSERT INTO sessions (id, name, host, port, username, password, private_key, group_name, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&data.name)
    .bind(&data.host)
    .bind(data.port)
    .bind(&data.username)
    .bind(&data.password)
    .bind(&data.private_key)
    .bind(&data.group_name)
    .bind(&now)
    .bind(&now)
    .execute(&self.pool)
    .await?;

        self.get(&id).await
    }

    async fn update(&self, data: UpdateSession) -> Result<Session> {
        let now = now_str();

        sqlx::query(
        "UPDATE sessions SET name=?, host=?, port=?, username=?, password=?, private_key=?, group_name=?, updated_at=? \
         WHERE id=?"
    )
    .bind(&data.name)
    .bind(&data.host)
    .bind(data.port)
    .bind(&data.username)
    .bind(&data.password)
    .bind(&data.private_key)
    .bind(&data.group_name)
    .bind(&now)
    .bind(&data.id)
    .execute(&self.pool)
    .await?;

        self.get(&data.id).await
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn import_if_empty(&self, sessions: Vec<Session>) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
            .fetch_one(&mut *tx)
            .await?;
        if count != 0 {
            anyhow::bail!("SQLite 目标存储非空，复制已取消");
        }

        for session in sessions {
            sqlx::query(
                "INSERT INTO sessions (id, name, host, port, username, password, private_key, group_name, created_at, updated_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(session.id)
            .bind(session.name)
            .bind(session.host)
            .bind(session.port)
            .bind(session.username)
            .bind(session.password)
            .bind(session.private_key)
            .bind(session.group_name)
            .bind(session.created_at)
            .bind(session.updated_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
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
}
