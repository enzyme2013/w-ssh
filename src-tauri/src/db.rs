use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use std::str::FromStr;
use anyhow::Result;
use uuid::Uuid;
use crate::models::{Session, CreateSession, UpdateSession};

pub async fn init_db(db_path: &str) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(opts).await?;

    sqlx::query(r#"
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
    "#)
    .execute(&pool)
    .await?;

    Ok(pool)
}

fn now_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
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

pub async fn get_sessions(pool: &SqlitePool) -> Result<Vec<Session>> {
    let rows = sqlx::query(
        "SELECT id, name, host, port, username, password, private_key, group_name, created_at, updated_at \
         FROM sessions ORDER BY group_name, name"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(row_to_session).collect())
}

pub async fn get_session_by_id(pool: &SqlitePool, id: &str) -> Result<Session> {
    let row = sqlx::query(
        "SELECT id, name, host, port, username, password, private_key, group_name, created_at, updated_at \
         FROM sessions WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(row_to_session(&row))
}

pub async fn create_session(pool: &SqlitePool, data: CreateSession) -> Result<Session> {
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
    .execute(pool)
    .await?;

    get_session_by_id(pool, &id).await
}

pub async fn update_session(pool: &SqlitePool, data: UpdateSession) -> Result<Session> {
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
    .execute(pool)
    .await?;

    get_session_by_id(pool, &data.id).await
}

pub async fn delete_session(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
