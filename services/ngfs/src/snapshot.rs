use crate::schema::{Cid, SnapshotV1};
use blake3::Hasher;
use serde_cbor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SnapshotError {
    #[error("Invalid mount point: {0}")]
    InvalidMountPoint(String),
    #[error("Mount not found: {0}")]
    MountNotFound(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Identity error: {0}")]
    IdentityError(String),
    #[error("KeyVault error: {0}")]
    KeyVaultError(String),
}

pub struct SnapshotBuilder {
    vclock_counter: std::sync::atomic::AtomicU64,
}

impl SnapshotBuilder {
    pub fn new() -> Self {
        Self {
            vclock_counter: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn build_snapshot(
        &self,
        mount_point: &str,
        root_cid: &Cid,
        signer_did: &str,
    ) -> Result<SnapshotV1, SnapshotError> {
        self.validate_mount_point(mount_point)?;
        
        let created_vclock = self.vclock_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        let mut snapshot = SnapshotV1 {
            snap_id: String::new(),
            root_cid: root_cid.clone(),
            created_vclock,
            signer_did: signer_did.to_string(),
        };
        
        let snap_id = self.compute_snapshot_id(&snapshot)?;
        snapshot.snap_id = snap_id;
        
        Ok(snapshot)
    }

    pub fn verify_snapshot(&self, snapshot: &SnapshotV1) -> Result<bool, SnapshotError> {
        let expected_id = self.compute_snapshot_id(snapshot)?;
        Ok(snapshot.snap_id == expected_id)
    }

    fn validate_mount_point(&self, mount_point: &str) -> Result<(), SnapshotError> {
        if mount_point.is_empty() {
            return Err(SnapshotError::InvalidMountPoint("empty mount point".to_string()));
        }
        
        if !mount_point.starts_with("/ro/") {
            return Err(SnapshotError::InvalidMountPoint(
                "mount point must be under /ro/".to_string(),
            ));
        }
        
        Ok(())
    }

    fn compute_snapshot_id(&self, snapshot: &SnapshotV1) -> Result<String, SnapshotError> {
        let mut temp_snapshot = snapshot.clone();
        temp_snapshot.snap_id = String::new();
        
        let cbor = serde_cbor::to_vec(&temp_snapshot)
            .map_err(|e| SnapshotError::SerializationFailed(format!("CBOR serialization failed: {}", e)))?;
        
        let mut hasher = Hasher::new();
        hasher.update(&cbor);
        let hash = hasher.finalize();
        
        let mut multihash = Vec::new();
        multihash.push(0x1f); 
        multihash.push(32);   
        multihash.extend_from_slice(hash.as_bytes());
        
        let cid = bs58::encode(multihash).into_string();
        Ok(format!("bafy{}", &cid[4..]))
    }

    pub fn get_vclock(&self) -> u64 {
        self.vclock_counter.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set_vclock(&self, value: u64) {
        self.vclock_counter.store(value, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Default for SnapshotBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_snapshot() {
        let builder = SnapshotBuilder::new();
        let root_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        let signer_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        
        let snapshot = builder.build_snapshot("/ro/test", &root_cid.to_string(), signer_did).unwrap();
        
        assert_eq!(snapshot.root_cid, root_cid);
        assert_eq!(snapshot.signer_did, signer_did);
        assert!(!snapshot.snap_id.is_empty());
        assert_eq!(snapshot.created_vclock, 0);
    }

    #[test]
    fn test_verify_snapshot() {
        let builder = SnapshotBuilder::new();
        let root_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        let signer_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        
        let snapshot = builder.build_snapshot("/ro/test", &root_cid.to_string(), signer_did).unwrap();
        let is_valid = builder.verify_snapshot(&snapshot).unwrap();
        
        assert!(is_valid);
    }

    #[test]
    fn test_invalid_mount_point() {
        let builder = SnapshotBuilder::new();
        let root_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        let signer_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        
        let result = builder.build_snapshot("/tmp/test", &root_cid.to_string(), signer_did);
        assert!(matches!(result, Err(SnapshotError::InvalidMountPoint(_))));
    }

    #[test]
    fn test_vclock_increment() {
        let builder = SnapshotBuilder::new();
        builder.set_vclock(100);
        
        let root_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        let signer_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        
        let snapshot1 = builder.build_snapshot("/ro/test1", &root_cid.to_string(), signer_did).unwrap();
        let snapshot2 = builder.build_snapshot("/ro/test2", &root_cid.to_string(), signer_did).unwrap();
        
        assert_eq!(snapshot1.created_vclock, 100);
        assert_eq!(snapshot2.created_vclock, 101);
    }

    #[test]
    fn test_snapshot_id_deterministic() {
        let builder = SnapshotBuilder::new();
        let root_cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        let signer_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        
        builder.set_vclock(42);
        
        let snapshot1 = builder.build_snapshot("/ro/test", &root_cid.to_string(), signer_did).unwrap();
        let snapshot2 = builder.build_snapshot("/ro/test", &root_cid.to_string(), signer_did).unwrap();
        
        assert_eq!(snapshot1.snap_id, snapshot2.snap_id);
    }
}
