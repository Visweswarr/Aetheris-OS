use crate::cas::CasIndex;
use crate::enc::NgfsEncryption;
use crate::manifest::FileManifestV1;
use crate::schema::{Cid, EncHeaderV1};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("CAS error: {0}")]
    CasError(String),
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("Invalid offset: {0}")]
    InvalidOffset(u64),
    #[error("Invalid length: {0}")]
    InvalidLength(usize),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Chunk not found: {0}")]
    ChunkNotFound(String),
}

pub struct FileReader {
    cas: CasIndex,
    encryption: NgfsEncryption,
}

impl FileReader {
    pub fn new(cas: CasIndex, encryption: NgfsEncryption) -> Self {
        Self { cas, encryption }
    }

    pub fn read_file(
        &self,
        file_cid: &Cid,
        offset: u64,
        len: usize,
    ) -> Result<Vec<u8>, ReadError> {
        let manifest = self.get_file_manifest(file_cid)?;
        
        if offset >= manifest.total_size {
            return Err(ReadError::InvalidOffset(offset));
        }
        
        if len == 0 {
            return Ok(Vec::new());
        }
        
        let max_read = (manifest.total_size - offset) as usize;
        let actual_len = std::cmp::min(len, max_read);
        
        let mut result = Vec::with_capacity(actual_len);
        let mut current_offset = offset;
        let mut remaining = actual_len;
        
        for (chunk_cid, chunk_len) in &manifest.chunks {
            if current_offset >= *chunk_len as u64 {
                current_offset -= *chunk_len as u64;
                continue;
            }
            
            let chunk_data = self.read_and_decrypt_chunk(chunk_cid)?;
            let chunk_start = current_offset as usize;
            let chunk_end = std::cmp::min(chunk_start + remaining, chunk_data.len());
            let chunk_slice = &chunk_data[chunk_start..chunk_end];
            
            result.extend_from_slice(chunk_slice);
            remaining -= chunk_slice.len();
            
            if remaining == 0 {
                break;
            }
            
            current_offset = 0;
        }
        
        Ok(result)
    }

    pub fn read_file_range(
        &self,
        file_cid: &Cid,
        start: u64,
        end: u64,
    ) -> Result<Vec<u8>, ReadError> {
        if start >= end {
            return Err(ReadError::InvalidOffset(start));
        }
        
        let len = (end - start) as usize;
        self.read_file(file_cid, start, len)
    }

    fn get_file_manifest(&self, file_cid: &Cid) -> Result<FileManifestV1, ReadError> {
        let data = self.cas.retrieve(file_cid)
            .map_err(|e| ReadError::CasError(format!("failed to retrieve file manifest: {}", e)))?;
        
        serde_cbor::from_slice(&data)
            .map_err(|e| ReadError::InvalidManifest(format!("invalid file manifest CBOR: {}", e)))
    }

    fn read_and_decrypt_chunk(&self, chunk_cid: &Cid) -> Result<Vec<u8>, ReadError> {
        let encrypted_data = self.cas.retrieve(chunk_cid)
            .map_err(|e| ReadError::CasError(format!("failed to retrieve chunk: {}", e)))?;
        
        if encrypted_data.len() < 256 {
            return Err(ReadError::InvalidManifest("chunk too small for header".to_string()));
        }
        
        let header_size = encrypted_data.len() - 256;
        let header_data = &encrypted_data[..header_size];
        let encrypted_chunk = &encrypted_data[header_size..];
        
        let header: EncHeaderV1 = serde_cbor::from_slice(header_data)
            .map_err(|e| ReadError::InvalidManifest(format!("invalid header CBOR: {}", e)))?;
        
        let associated_data = self.build_associated_data(&header, chunk_cid)?;
        
        let decrypted = self.encryption.open_envelope(&header.kid, &encrypted_chunk, &associated_data)
            .map_err(|e| ReadError::DecryptionFailed(format!("failed to decrypt chunk: {}", e)))?;
        
        Ok(decrypted)
    }

    fn build_associated_data(&self, header: &EncHeaderV1, chunk_cid: &Cid) -> Result<Vec<u8>, ReadError> {
        let mut ad = Vec::new();
        
        ad.extend_from_slice(&header.alg.to_le_bytes());
        ad.extend_from_slice(&header.kid.as_bytes());
        ad.extend_from_slice(chunk_cid.as_bytes());
        
        Ok(ad)
    }

    pub fn get_file_size(&self, file_cid: &Cid) -> Result<u64, ReadError> {
        let manifest = self.get_file_manifest(file_cid)?;
        Ok(manifest.total_size)
    }

    pub fn get_chunk_count(&self, file_cid: &Cid) -> Result<usize, ReadError> {
        let manifest = self.get_file_manifest(file_cid)?;
        Ok(manifest.chunks.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cas::CasIndex;
    use crate::enc::NgfsEncryption;
    use crate::manifest::FileManifestV1;
    use crate::schema::{EncHeaderV1, EncEnvelopeV1};
    use tempfile::tempdir;

    fn create_test_components() -> (CasIndex, NgfsEncryption) {
        let temp_dir = tempdir().unwrap();
        let cas = CasIndex::new(temp_dir.path().join("test_cas"));
        let encryption = NgfsEncryption::new();
        (cas, encryption)
    }

    #[test]
    fn test_read_file_basic() {
        let (cas, encryption) = create_test_components();
        let reader = FileReader::new(cas, encryption);
        
        let result = reader.read_file(&"test_cid".to_string(), 0, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_invalid_offset() {
        let (cas, encryption) = create_test_components();
        let reader = FileReader::new(cas, encryption);
        
        let result = reader.read_file(&"test_cid".to_string(), 1000, 100);
        assert!(matches!(result, Err(ReadError::InvalidOffset(_))));
    }

    #[test]
    fn test_read_file_zero_length() {
        let (cas, encryption) = create_test_components();
        let reader = FileReader::new(cas, encryption);
        
        let result = reader.read_file(&"test_cid".to_string(), 0, 0);
        assert_eq!(result.unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_read_file_range_invalid() {
        let (cas, encryption) = create_test_components();
        let reader = FileReader::new(cas, encryption);
        
        let result = reader.read_file_range(&"test_cid".to_string(), 100, 50);
        assert!(matches!(result, Err(ReadError::InvalidOffset(_))));
    }
}
