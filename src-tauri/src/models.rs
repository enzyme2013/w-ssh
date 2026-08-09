use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    Password,
    PrivateKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CredentialState {
    #[default]
    None,
    Stored,
    LegacyPlaintext,
    NeedsRebind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialKind {
    Password,
    PrivateKeyPassphrase,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub private_key: Option<String>,
    pub auth_method: AuthMethod,
    pub credential_state: CredentialState,
    pub icon: Option<String>,
    pub group_name: Option<String>,
    pub group_icon: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSession {
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub private_key: Option<String>,
    pub auth_method: AuthMethod,
    pub icon: Option<String>,
    pub group_name: Option<String>,
    pub group_icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSession {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub private_key: Option<String>,
    pub auth_method: AuthMethod,
    pub icon: Option<String>,
    pub group_name: Option<String>,
    pub group_icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGroup {
    pub current_name: String,
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetCredentialRequest {
    pub session_id: String,
    pub kind: CredentialKind,
    pub secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectSecret {
    pub kind: CredentialKind,
    pub secret: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LegacyCredentialSummary {
    pub count: i64,
    pub session_ids: Vec<String>,
}
