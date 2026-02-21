use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub group_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSession {
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub group_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSession {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub group_name: Option<String>,
}
