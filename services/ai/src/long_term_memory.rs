//! Long-term memory backed by NGFS-like KV on the filesystem
//! Keys are stored under <root>/kv/<key>.cbor and snapshots under <root>/snapshots/<id>.cbor

use crate::error::{AiError, AiResult};
use serde::{Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};

#[async_trait::async_trait]
pub trait LongTermMemory: Send + Sync {
    async fn put<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> AiResult<()>;
    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> AiResult<Option<T>>;
    async fn list(&self, prefix: &str) -> AiResult<Vec<String>>;
}

pub struct NgfsKv {
    pub root: PathBuf,
}

impl NgfsKv {
    pub fn new<P: AsRef<Path>>(root: P) -> Self { Self { root: root.as_ref().to_path_buf() } }

    fn kv_path(&self, key: &str) -> PathBuf {
        let mut p = self.root.clone();
        p.push("kv");
        // convert slashes in key to path separators
        for part in key.split('/') { p.push(part); }
        p.set_extension("cbor");
        p
    }

    fn snapshots_dir(&self) -> PathBuf {
        let mut p = self.root.clone();
        p.push("snapshots");
        p
    }

    pub async fn write_snapshot_bytes(&self, id: &str, bytes: &[u8]) -> AiResult<()> {
        let mut p = self.snapshots_dir();
        tokio::fs::create_dir_all(&p).await.ok();
        p.push(format!("{}.cbor", id));
        tokio::fs::write(p, bytes).await.map_err(|e| AiError::io(e.to_string()))
    }

    pub async fn read_snapshot_bytes(&self, id: &str) -> AiResult<Vec<u8>> {
        let mut p = self.snapshots_dir();
        p.push(format!("{}.cbor", id));
        let data = tokio::fs::read(p).await.map_err(|e| AiError::io(e.to_string()))?;
        Ok(data)
    }
}

#[async_trait::async_trait]
impl LongTermMemory for NgfsKv {
    async fn put<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> AiResult<()> {
        let path = self.kv_path(key);
        if let Some(parent) = path.parent() { tokio::fs::create_dir_all(parent).await.ok(); }
        let mut buf = Vec::new();
        ciborium::ser::into_writer(value, &mut buf).map_err(|e| AiError::serialization(e.to_string()))?;
        tokio::fs::write(path, buf).await.map_err(|e| AiError::io(e.to_string()))
    }

    async fn get<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> AiResult<Option<T>> {
        let path = self.kv_path(key);
        if !path.exists() { return Ok(None); }
        let data = tokio::fs::read(path).await.map_err(|e| AiError::io(e.to_string()))?;
        let val: T = ciborium::de::from_reader(data.as_slice()).map_err(|e| AiError::deserialization(e.to_string()))?;
        Ok(Some(val))
    }

    async fn list(&self, prefix: &str) -> AiResult<Vec<String>> {
        let mut dir = self.root.clone();
        dir.push("kv");
        for part in prefix.split('/') { dir.push(part); }
        let mut keys = Vec::new();
        if let Ok(mut rd) = tokio::fs::read_dir(&dir).await {
            while let Ok(Some(entry)) = rd.next_entry().await {
                if let Some(name) = entry.file_name().to_str() {
                    keys.push(name.to_string());
                }
            }
        }
        Ok(keys)
    }
}
