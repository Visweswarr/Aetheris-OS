use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cid::{Cid, cid_from_bytes, verify_chunk, compute_checksum, verify_checksum, CidError};
use crate::schema::{CASChunkV1, ContentId, ContentType};

/// CAS storage configuration
pub const NGFS_SEGMENT_SIZE: u64 = 8 * 1024 * 1024; // 8 MiB
pub const NGFS_MAX_CHUNK_SIZE: usize = 256 * 1024; // 256 KiB
pub const NGFS_MAX_READ_SIZE: usize = 64 * 1024; // 64 KiB
pub const NGFS_INDEX_FILENAME: &str = "ngfs.idx";
pub const NGFS_DATA_FILENAME: &str = "ngfs.dat";

/// Chunk header for CAS storage
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkHeader {
    /// Content identifier hash
    pub cid_hash: [u8; 32],
    /// Chunk offset within segment
    pub offset: u64,
    /// Chunk length in bytes
    pub length: u32,
    /// Chunk checksum
    pub checksum: u32,
    /// Timestamp (virtual clock)
    pub timestamp: u64,
}

impl ChunkHeader {
    /// Create a new chunk header
    pub fn new(cid_hash: [u8; 32], offset: u64, length: u32, timestamp: u64) -> Self {
        Self {
            cid_hash,
            offset,
            length,
            checksum: 0, // Will be computed
            timestamp,
        }
    }

    /// Compute the header size
    pub fn size() -> usize {
        32 + 8 + 4 + 4 + 8 // cid_hash + offset + length + checksum + timestamp
    }

    /// Serialize header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::size());
        bytes.extend_from_slice(&self.cid_hash);
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        bytes.extend_from_slice(&self.length.to_le_bytes());
        bytes.extend_from_slice(&self.checksum.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes
    }

    /// Deserialize header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CasError> {
        if bytes.len() < Self::size() {
            return Err(CasError::InvalidHeader);
        }

        let mut cid_hash = [0u8; 32];
        cid_hash.copy_from_slice(&bytes[0..32]);

        let offset = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
        let length = u32::from_le_bytes(bytes[40..44].try_into().unwrap());
        let checksum = u32::from_le_bytes(bytes[44..48].try_into().unwrap());
        let timestamp = u64::from_le_bytes(bytes[48..56].try_into().unwrap());

        Ok(Self {
            cid_hash,
            offset,
            length,
            checksum,
            timestamp,
        })
    }
}

/// Segment trailer for crash recovery
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentTrailer {
    /// Segment ID
    pub segment_id: u32,
    /// Number of chunks in segment
    pub chunk_count: u32,
    /// Segment checksum
    pub checksum: u32,
    /// Magic number for validation
    pub magic: u32,
}

impl SegmentTrailer {
    /// Magic number for segment validation
    pub const MAGIC: u32 = 0x4E474653; // "NGFS"

    /// Create a new segment trailer
    pub fn new(segment_id: u32, chunk_count: u32, checksum: u32) -> Self {
        Self {
            segment_id,
            chunk_count,
            checksum,
            magic: Self::MAGIC,
        }
    }

    /// Serialize trailer to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend_from_slice(&self.segment_id.to_le_bytes());
        bytes.extend_from_slice(&self.chunk_count.to_le_bytes());
        bytes.extend_from_slice(&self.checksum.to_le_bytes());
        bytes.extend_from_slice(&self.magic.to_le_bytes());
        bytes
    }

    /// Deserialize trailer from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CasError> {
        if bytes.len() < 16 {
            return Err(CasError::InvalidTrailer);
        }

        let segment_id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let chunk_count = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let checksum = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let magic = u32::from_le_bytes(bytes[12..16].try_into().unwrap());

        if magic != Self::MAGIC {
            return Err(CasError::InvalidMagic);
        }

        Ok(Self {
            segment_id,
            chunk_count,
            checksum,
            magic,
        })
    }
}

/// Index entry for chunk lookup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    /// Segment ID where chunk is stored
    pub segment_id: u32,
    /// Offset within segment
    pub offset: u64,
    /// Chunk length
    pub length: u32,
    /// Timestamp when chunk was written
    pub timestamp: u64,
}

/// CAS Index for chunk lookup
pub struct CasIndex {
    /// In-memory index mapping CID to location
    map: BTreeMap<[u8; 32], IndexEntry>,
    /// Index file path
    path: PathBuf,
    /// Index file for persistence
    file: Option<File>,
}

impl CasIndex {
    /// Create a new CAS index
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            map: BTreeMap::new(),
            path: path.as_ref().to_path_buf(),
            file: None,
        }
    }

    /// Load index from file
    pub fn load(&mut self) -> Result<(), CasError> {
        if !self.path.exists() {
            return Ok(()); // New index
        }

        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.path)
            .map_err(|e| CasError::IoError(e.to_string()))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| CasError::IoError(e.to_string()))?;

        // Parse CBOR index
        let index_data: BTreeMap<String, (u32, u64, u32, u64)> = serde_cbor::from_slice(&buffer)
            .map_err(|e| CasError::IndexCorruption(e.to_string()))?;

        for (cid_hex, (segment_id, offset, length, timestamp)) in index_data {
            let cid_bytes = hex::decode(&cid_hex)
                .map_err(|e| CasError::IndexCorruption(e.to_string()))?;
            
            if cid_bytes.len() == 32 {
                let mut cid_hash = [0u8; 32];
                cid_hash.copy_from_slice(&cid_bytes);
                
                self.map.insert(cid_hash, IndexEntry {
                    segment_id,
                    offset,
                    length,
                    timestamp,
                });
            }
        }

        Ok(())
    }

    /// Save index to file atomically
    pub fn save(&self) -> Result<(), CasError> {
        // Create temporary index data
        let mut index_data = BTreeMap::new();
        for (cid_hash, entry) in &self.map {
            let cid_hex = hex::encode(cid_hash);
            index_data.insert(cid_hex, (entry.segment_id, entry.offset, entry.length, entry.timestamp));
        }

        // Serialize to CBOR
        let cbor_data = serde_cbor::to_vec(&index_data)
            .map_err(|e| CasError::IndexCorruption(e.to_string()))?;

        // Write to temporary file
        let temp_path = self.path.with_extension("tmp");
        let mut temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .map_err(|e| CasError::IoError(e.to_string()))?;

        temp_file.write_all(&cbor_data)
            .map_err(|e| CasError::IoError(e.to_string()))?;
        temp_file.flush()
            .map_err(|e| CasError::IoError(e.to_string()))?;

        // Atomic rename
        std::fs::rename(&temp_path, &self.path)
            .map_err(|e| CasError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Insert a chunk into the index
    pub fn insert(&mut self, cid_hash: [u8; 32], entry: IndexEntry) {
        self.map.insert(cid_hash, entry);
    }

    /// Look up a chunk by CID
    pub fn get(&self, cid_hash: &[u8; 32]) -> Option<&IndexEntry> {
        self.map.get(cid_hash)
    }

    /// Get the total number of chunks
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// CAS Segment Store
pub struct CasSegmentStore {
    /// Data file for segments
    data_file: File,
    /// Current segment ID
    current_segment: u32,
    /// Current offset within segment
    current_offset: u64,
    /// Index for chunk lookup
    index: CasIndex,
    /// Data file path
    data_path: PathBuf,
    /// Index path
    index_path: PathBuf,
}

impl CasSegmentStore {
    /// Create a new CAS segment store
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self, CasError> {
        let base_path = base_path.as_ref();
        let data_path = base_path.join(NGFS_DATA_FILENAME);
        let index_path = base_path.join(NGFS_INDEX_FILENAME);

        // Create base directory if it doesn't exist
        if let Some(parent) = base_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CasError::IoError(e.to_string()))?;
        }

        // Open or create data file
        let data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&data_path)
            .map_err(|e| CasError::IoError(e.to_string()))?;

        // Create index
        let mut index = CasIndex::new(&index_path);
        index.load()?;

        // Determine current segment and offset
        let (current_segment, current_offset) = Self::determine_current_position(&data_file)?;

        Ok(Self {
            data_file,
            current_segment,
            current_offset,
            index,
            data_path,
            index_path,
        })
    }

    /// Determine current segment and offset from data file
    fn determine_current_position(file: &File) -> Result<(u32, u64), CasError> {
        let metadata = file.metadata()
            .map_err(|e| CasError::IoError(e.to_string()))?;
        
        let file_size = metadata.len();
        let current_segment = (file_size / NGFS_SEGMENT_SIZE) as u32;
        let current_offset = file_size % NGFS_SEGMENT_SIZE;
        
        Ok((current_segment, current_offset))
    }

    /// Get current virtual clock timestamp
    fn get_virtual_clock() -> u64 {
        // Use system time as virtual clock for now
        // In a real implementation, this would come from the kernel
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Append a chunk to the store
    pub fn append_chunk(&mut self, bytes: &[u8]) -> Result<Cid, CasError> {
        if bytes.len() > NGFS_MAX_CHUNK_SIZE {
            return Err(CasError::ChunkTooLarge);
        }

        // Compute CID
        let cid = cid_from_bytes(bytes)?;
        let cid_hash = cid.hash;

        // Check if chunk already exists
        if self.index.get(&cid_hash).is_some() {
            return Ok(cid); // Deduplication
        }

        // Check if we need to start a new segment
        if self.current_offset + ChunkHeader::size() as u64 + bytes.len() as u64 > NGFS_SEGMENT_SIZE {
            self.finalize_current_segment()?;
            self.start_new_segment()?;
        }

        // Write chunk header
        let header = ChunkHeader::new(
            cid_hash,
            self.current_offset,
            bytes.len() as u32,
            Self::get_virtual_clock(),
        );

        let header_bytes = header.to_bytes();
        self.data_file.write_all(&header_bytes)
            .map_err(|e| CasError::IoError(e.to_string()))?;

        // Write chunk data
        self.data_file.write_all(bytes)
            .map_err(|e| CasError::IoError(e.to_string()))?;

        // Update index
        let entry = IndexEntry {
            segment_id: self.current_segment,
            offset: self.current_offset,
            length: bytes.len() as u32,
            timestamp: header.timestamp,
        };
        self.index.insert(cid_hash, entry);

        // Update current offset
        self.current_offset += header_bytes.len() as u64 + bytes.len() as u64;

        // Periodically save index
        if self.index.len() % 1000 == 0 {
            self.index.save()?;
        }

        Ok(cid)
    }

    /// Read a chunk by CID
    pub fn read_chunk(&mut self, cid: &Cid) -> Result<Vec<u8>, CasError> {
        let cid_hash = cid.hash;
        
        // Look up chunk in index
        let entry = self.index.get(&cid_hash)
            .ok_or(CasError::ChunkNotFound)?;

        // Seek to chunk location
        let chunk_offset = entry.segment_id as u64 * NGFS_SEGMENT_SIZE + entry.offset;
        self.data_file.seek(SeekFrom::Start(chunk_offset))
            .map_err(|e| CasError::IoError(e.to_string()))?;

        // Read and parse header
        let mut header_bytes = vec![0u8; ChunkHeader::size()];
        self.data_file.read_exact(&mut header_bytes)
            .map_err(|e| CasError::IoError(e.to_string()))?;

        let header = ChunkHeader::from_bytes(&header_bytes)?;
        
        // Verify header matches expected values
        if header.cid_hash != cid_hash || header.length != entry.length {
            return Err(CasError::HeaderMismatch);
        }

        // Read chunk data
        let mut chunk_data = vec![0u8; header.length as usize];
        self.data_file.read_exact(&mut chunk_data)
            .map_err(|e| CasError::IoError(e.to_string()))?;

        // Verify chunk integrity
        if !verify_chunk(&chunk_data, cid) {
            return Err(CasError::ChunkCorruption);
        }

        Ok(chunk_data)
    }

    /// Finalize current segment
    fn finalize_current_segment(&mut self) -> Result<(), CasError> {
        if self.current_offset == 0 {
            return Ok(()); // Empty segment
        }

        // Write segment trailer
        let trailer = SegmentTrailer::new(
            self.current_segment,
            self.get_chunk_count_in_segment()?,
            self.compute_segment_checksum()?,
        );

        let trailer_bytes = trailer.to_bytes();
        self.data_file.write_all(&trailer_bytes)
            .map_err(|e| CasError::IoError(e.to_string()))?;

        // Flush to disk
        self.data_file.flush()
            .map_err(|e| CasError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Start a new segment
    fn start_new_segment(&mut self) -> Result<(), CasError> {
        self.current_segment += 1;
        self.current_offset = 0;
        Ok(())
    }

    /// Get chunk count in current segment
    fn get_chunk_count_in_segment(&self) -> Result<u32, CasError> {
        // Count chunks in current segment by scanning index
        let mut count = 0;
        for entry in self.index.map.values() {
            if entry.segment_id == self.current_segment {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Compute checksum for current segment
    fn compute_segment_checksum(&self) -> Result<u32, CasError> {
        // For now, return a simple checksum
        // In a real implementation, this would compute a hash of the segment
        Ok(self.current_segment as u32)
    }

    /// Recover from crash by scanning segments
    pub fn recover(&mut self) -> Result<(), CasError> {
        // Clear current index
        self.index.map.clear();

        // Scan data file and rebuild index
        self.scan_and_rebuild_index()?;

        // Load index
        self.index.load()?;

        // Determine current position
        let (current_segment, current_offset) = Self::determine_current_position(&self.data_file)?;
        self.current_segment = current_segment;
        self.current_offset = current_offset;

        Ok(())
    }

    /// Scan data file and rebuild index
    fn scan_and_rebuild_index(&mut self) -> Result<(), CasError> {
        self.data_file.seek(SeekFrom::Start(0))
            .map_err(|e| CasError::IoError(e.to_string()))?;

        let mut offset = 0u64;
        let mut segment_id = 0u32;
        let mut chunk_count = 0u32;

        loop {
            // Try to read header
            let mut header_bytes = vec![0u8; ChunkHeader::size()];
            match self.data_file.read_exact(&mut header_bytes) {
                Ok(_) => {},
                Err(_) => break, // End of file
            }

            // Parse header
            let header = match ChunkHeader::from_bytes(&header_bytes) {
                Ok(h) => h,
                Err(_) => break, // Invalid header, end of segment
            };

            // Check if we're starting a new segment
            if offset >= NGFS_SEGMENT_SIZE {
                segment_id += 1;
                offset = 0;
                chunk_count = 0;
            }

            // Read chunk data
            let mut chunk_data = vec![0u8; header.length as usize];
            if let Err(_) = self.data_file.read_exact(&mut chunk_data) {
                break; // Incomplete chunk, end of segment
            }

            // Verify chunk integrity
            let cid = Cid::new(header.cid_hash);
            if verify_chunk(&chunk_data, &cid) {
                // Add to index
                let entry = IndexEntry {
                    segment_id,
                    offset,
                    length: header.length,
                    timestamp: header.timestamp,
                };
                self.index.insert(header.cid_hash, entry);
                chunk_count += 1;
            }

            // Move to next chunk
            offset += header_bytes.len() as u64 + header.length as u64;
        }

        Ok(())
    }

    /// Get store statistics
    pub fn get_stats(&self) -> CasStats {
        CasStats {
            total_chunks: self.index.len(),
            current_segment: self.current_segment,
            current_offset: self.current_offset,
            total_segments: (self.current_segment + 1) as usize,
        }
    }

    /// Flush all data to disk
    pub fn flush(&mut self) -> Result<(), CasError> {
        self.finalize_current_segment()?;
        self.index.save()?;
        self.data_file.flush()
            .map_err(|e| CasError::IoError(e.to_string()))?;
        Ok(())
    }
}

/// CAS storage statistics
#[derive(Debug, Clone)]
pub struct CasStats {
    /// Total number of chunks stored
    pub total_chunks: usize,
    /// Current segment ID
    pub current_segment: u32,
    /// Current offset within segment
    pub current_offset: u64,
    /// Total number of segments
    pub total_segments: usize,
}

/// CAS-related errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CasError {
    /// Chunk size exceeds maximum allowed
    ChunkTooLarge,
    /// Chunk not found in store
    ChunkNotFound,
    /// Chunk data is corrupted
    ChunkCorruption,
    /// Header data is invalid
    InvalidHeader,
    /// Trailer data is invalid
    InvalidTrailer,
    /// Invalid magic number
    InvalidMagic,
    /// Header doesn't match expected values
    HeaderMismatch,
    /// Index data is corrupted
    IndexCorruption(String),
    /// I/O error occurred
    IoError(String),
}

impl std::fmt::Display for CasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CasError::ChunkTooLarge => write!(f, "Chunk size exceeds maximum allowed"),
            CasError::ChunkNotFound => write!(f, "Chunk not found in store"),
            CasError::ChunkCorruption => write!(f, "Chunk data is corrupted"),
            CasError::InvalidHeader => write!(f, "Invalid chunk header"),
            CasError::InvalidTrailer => write!(f, "Invalid segment trailer"),
            CasError::InvalidMagic => write!(f, "Invalid magic number"),
            CasError::HeaderMismatch => write!(f, "Header doesn't match expected values"),
            CasError::IndexCorruption(msg) => write!(f, "Index corruption: {}", msg),
            CasError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for CasError {}

impl From<CidError> for CasError {
    fn from(err: CidError) -> Self {
        match err {
            CidError::ChunkTooLarge => CasError::ChunkTooLarge,
            CidError::InvalidFormat => CasError::InvalidHeader,
            CidError::HashComputationFailed => CasError::ChunkCorruption,
            CidError::ChecksumMismatch => CasError::ChunkCorruption,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_chunk_header_serialization() {
        let header = ChunkHeader::new(
            [1u8; 32],
            1024,
            512,
            12345,
        );

        let bytes = header.to_bytes();
        let parsed = ChunkHeader::from_bytes(&bytes).unwrap();

        assert_eq!(header, parsed);
    }

    #[test]
    fn test_segment_trailer_serialization() {
        let trailer = SegmentTrailer::new(1, 100, 0x12345678);

        let bytes = trailer.to_bytes();
        let parsed = SegmentTrailer::from_bytes(&bytes).unwrap();

        assert_eq!(trailer, parsed);
    }

    #[test]
    fn test_cas_store_creation() {
        let temp_dir = tempdir().unwrap();
        let store = CasSegmentStore::new(temp_dir.path());
        assert!(store.is_ok());
    }

    #[test]
    fn test_chunk_append_and_read() {
        let temp_dir = tempdir().unwrap();
        let mut store = CasSegmentStore::new(temp_dir.path()).unwrap();

        let test_data = b"Hello, NGFS CAS!";
        let cid = store.append_chunk(test_data).unwrap();

        let retrieved_data = store.read_chunk(&cid).unwrap();
        assert_eq!(test_data, retrieved_data.as_slice());
    }

    #[test]
    fn test_chunk_deduplication() {
        let temp_dir = tempdir().unwrap();
        let mut store = CasSegmentStore::new(temp_dir.path()).unwrap();

        let test_data = b"Duplicate chunk test";
        
        let cid1 = store.append_chunk(test_data).unwrap();
        let cid2 = store.append_chunk(test_data).unwrap();

        assert_eq!(cid1, cid2);
        
        let stats = store.get_stats();
        assert_eq!(stats.total_chunks, 1); // Only one unique chunk
    }

    #[test]
    fn test_chunk_size_limit() {
        let temp_dir = tempdir().unwrap();
        let mut store = CasSegmentStore::new(temp_dir.path()).unwrap();

        let large_data = vec![0u8; NGFS_MAX_CHUNK_SIZE + 1];
        let result = store.append_chunk(&large_data);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CasError::ChunkTooLarge);
    }

    #[test]
    fn test_store_recovery() {
        let temp_dir = tempdir().unwrap();
        let mut store = CasSegmentStore::new(temp_dir.path()).unwrap();

        // Add some chunks
        let data1 = b"Recovery test chunk 1";
        let data2 = b"Recovery test chunk 2";
        
        let cid1 = store.append_chunk(data1).unwrap();
        let cid2 = store.append_chunk(data2).unwrap();

        // Flush to disk
        store.flush().unwrap();

        // Create new store instance (simulating restart)
        let mut new_store = CasSegmentStore::new(temp_dir.path()).unwrap();
        
        // Verify chunks can still be read
        let retrieved1 = new_store.read_chunk(&cid1).unwrap();
        let retrieved2 = new_store.read_chunk(&cid2).unwrap();

        assert_eq!(data1, retrieved1.as_slice());
        assert_eq!(data2, retrieved2.as_slice());
    }
}
