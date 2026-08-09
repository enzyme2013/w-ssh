use anyhow::Result;
use async_trait::async_trait;

use crate::models::{CreateSession, Session, UpdateGroup, UpdateSession};

pub fn now_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after the Unix epoch")
        .as_secs()
        .to_string()
}

#[async_trait]
pub trait SessionStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Session>>;
    async fn get(&self, id: &str) -> Result<Session>;
    async fn create(&self, data: CreateSession) -> Result<Session>;
    async fn update(&self, data: UpdateSession) -> Result<Session>;
    async fn update_group(&self, data: UpdateGroup) -> Result<Vec<Session>>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn import_if_empty(&self, sessions: Vec<Session>) -> Result<()>;
    async fn take_notice(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
pub async fn assert_storage_contract(storage: &dyn SessionStorage) {
    let created = storage
        .create(CreateSession {
            name: "Contract Host".into(),
            host: "example.test".into(),
            port: 22,
            username: "tester".into(),
            private_key: Some("fixture-key-path".into()),
            auth_method: crate::models::AuthMethod::PrivateKey,
            icon: Some("server".into()),
            group_name: Some("contract".into()),
            group_icon: Some("folder".into()),
        })
        .await
        .expect("create should succeed");

    assert!(!created.id.is_empty());
    assert!(!created.created_at.is_empty());
    assert_eq!(storage.get(&created.id).await.unwrap(), created);
    assert_eq!(storage.list().await.unwrap(), vec![created.clone()]);

    let updated = storage
        .update(UpdateSession {
            id: created.id.clone(),
            name: "Updated Contract Host".into(),
            host: "updated.example.test".into(),
            port: 2202,
            username: "updated-user".into(),
            private_key: Some("updated-fixture-key-path".into()),
            auth_method: crate::models::AuthMethod::PrivateKey,
            icon: Some("cloud".into()),
            group_name: Some("updated-contract".into()),
            group_icon: Some("layers".into()),
        })
        .await
        .expect("update should succeed");

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.created_at, created.created_at);
    assert_eq!(storage.get(&created.id).await.unwrap(), updated);

    let renamed = storage
        .update_group(UpdateGroup {
            current_name: "updated-contract".into(),
            name: "renamed-contract".into(),
            icon: Some("briefcase".into()),
        })
        .await
        .expect("group update should succeed");
    assert_eq!(renamed.len(), 1);
    assert_eq!(renamed[0].group_name.as_deref(), Some("renamed-contract"));
    assert_eq!(renamed[0].group_icon.as_deref(), Some("briefcase"));

    storage
        .delete(&created.id)
        .await
        .expect("delete should succeed");
    assert!(storage.list().await.unwrap().is_empty());
    assert!(storage.get(&created.id).await.is_err());
}
