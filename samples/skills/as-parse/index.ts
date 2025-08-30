/**
 * NGFS Manifest & Proof Parser Skill (AssemblyScript)
 * 
 * This skill demonstrates parsing NGFS manifests and proofs,
 * emitting preview events for successful validation.
 */

// NGFS constants
const NGFS_MAX_NAME_LENGTH = 255;
const NGFS_MAX_DIR_ENTRIES = 65_535;
const NGFS_MAX_FILE_CHUNKS = 4_096;

// Entry kinds
const ENTRY_KIND_DIRECTORY = 0;
const ENTRY_KIND_FILE = 1;
const ENTRY_KIND_SYMLINK = 2;

// Content types
const CONTENT_TYPE_RAW = 0;
const CONTENT_TYPE_DIRECTORY = 1;
const CONTENT_TYPE_FILE_MANIFEST = 2;
const CONTENT_TYPE_SNAPSHOT = 3;
const CONTENT_TYPE_SYMLINK = 4;
const CONTENT_TYPE_SPECIAL = 5;

// Simple CBOR parsing for NGFS structures
// This is a minimal implementation for demonstration purposes

/**
 * Simple CBOR decoder for NGFS manifests
 */
class SimpleCBORDecoder {
  private data: Uint8Array;
  private offset: i32;

  constructor(data: Uint8Array) {
    this.data = data;
    this.offset = 0;
  }

  private readByte(): u8 {
    if (this.offset >= this.data.length) {
      throw new Error("Unexpected end of data");
    }
    return this.data[this.offset++];
  }

  private readLength(): u32 {
    const firstByte = this.readByte();
    const majorType = (firstByte >> 5) & 0x07;
    const additionalInfo = firstByte & 0x1F;

    if (additionalInfo < 24) {
      return additionalInfo;
    } else if (additionalInfo == 24) {
      return this.readByte();
    } else if (additionalInfo == 25) {
      return (this.readByte() << 8) | this.readByte();
    } else if (additionalInfo == 26) {
      return (this.readByte() << 24) | (this.readByte() << 16) | (this.readByte() << 8) | this.readByte();
    } else {
      throw new Error("Unsupported length encoding");
    }
  }

  private readString(): string {
    const length = this.readLength();
    const bytes = new Uint8Array(length);
    
    for (let i = 0; i < length; i++) {
      bytes[i] = this.readByte();
    }
    
    // Convert bytes to string (simplified)
    let result = "";
    for (let i = 0; i < length; i++) {
      result += String.fromCharCode(bytes[i]);
    }
    
    return result;
  }

  private readUint32(): u32 {
    const length = this.readLength();
    if (length != 4) {
      throw new Error("Expected 4 bytes for uint32");
    }
    
    return (this.readByte() << 24) | (this.readByte() << 16) | (this.readByte() << 8) | this.readByte();
  }

  private readUint64(): u64 {
    const length = this.readLength();
    if (length != 8) {
      throw new Error("Expected 8 bytes for uint64");
    }
    
    let result: u64 = 0;
    for (let i = 0; i < 8; i++) {
      result = (result << 8) | this.readByte();
    }
    
    return result;
  }

  private readArray(): Array<any> {
    const length = this.readLength();
    const result = new Array<any>();
    
    for (let i = 0; i < length; i++) {
      result.push(this.readValue());
    }
    
    return result;
  }

  private readMap(): Map<string, any> {
    const length = this.readLength();
    const result = new Map<string, any>();
    
    for (let i = 0; i < length; i++) {
      const key = this.readString();
      const value = this.readValue();
      result.set(key, value);
    }
    
    return result;
  }

  private readValue(): any {
    const firstByte = this.readByte();
    const majorType = (firstByte >> 5) & 0x07;
    
    // Push the byte back to read it again with length
    this.offset--;
    
    switch (majorType) {
      case 0: // Unsigned integer
        return this.readLength();
      case 1: // Negative integer
        this.readLength(); // Skip for now
        return -1;
      case 2: // Byte string
        this.readLength(); // Skip for now
        return new Uint8Array(0);
      case 3: // Text string
        return this.readString();
      case 4: // Array
        return this.readArray();
      case 5: // Map
        return this.readMap();
      case 6: // Tag
        this.readLength(); // Skip for now
        return this.readValue();
      case 7: // Simple/float
        const additionalInfo = firstByte & 0x1F;
        if (additionalInfo == 20) return false;
        if (additionalInfo == 21) return true;
        if (additionalInfo == 22) return null;
        if (additionalInfo == 23) return null; // undefined
        return this.readLength();
      default:
        throw new Error("Unknown CBOR major type");
    }
  }

  decode(): any {
    return this.readValue();
  }
}

/**
 * NGFS Content ID structure
 */
class ContentID {
  blake3Hash: Uint8Array;
  ipfsMultihash: Uint8Array | null;
  contentType: u8;

  constructor(blake3Hash: Uint8Array, contentType: u8) {
    this.blake3Hash = blake3Hash;
    this.ipfsMultihash = null;
    this.contentType = contentType;
  }

  static fromMap(map: Map<string, any>): ContentID {
    const blake3Hash = map.get("blake3_hash") as Uint8Array;
    const contentType = map.get("content_type") as u8;
    
    if (!blake3Hash || blake3Hash.length != 32) {
      throw new Error("Invalid blake3_hash");
    }
    
    return new ContentID(blake3Hash, contentType);
  }
}

/**
 * NGFS Entry structure
 */
class Entry {
  name: string;
  kind: u8;
  cid: ContentID;
  size: u64 | null;
  mode: u16 | null;

  constructor(name: string, kind: u8, cid: ContentID, size: u64 | null = null, mode: u16 | null = null) {
    this.name = name;
    this.kind = kind;
    this.cid = cid;
    this.size = size;
    this.mode = mode;
  }

  static fromMap(map: Map<string, any>): Entry {
    const name = map.get("name") as string;
    const kind = map.get("kind") as u8;
    const cidMap = map.get("cid") as Map<string, any>;
    const size = map.get("size") as u64 | null;
    const mode = map.get("mode") as u16 | null;
    
    if (!name || kind > 2) {
      throw new Error("Invalid entry data");
    }
    
    const cid = ContentID.fromMap(cidMap);
    return new Entry(name, kind, cid, size, mode);
  }
}

/**
 * NGFS Chunk Info structure
 */
class ChunkInfo {
  cid: ContentID;
  length: u32;

  constructor(cid: ContentID, length: u32) {
    this.cid = cid;
    this.length = length;
  }

  static fromMap(map: Map<string, any>): ChunkInfo {
    const cidMap = map.get("cid") as Map<string, any>;
    const length = map.get("length") as u32;
    
    if (length == 0) {
      throw new Error("Invalid chunk length");
    }
    
    const cid = ContentID.fromMap(cidMap);
    return new ChunkInfo(cid, length);
  }
}

/**
 * NGFS Directory Manifest structure
 */
class DirManifest {
  version: u16;
  entries: Array<Entry>;

  constructor(version: u16, entries: Array<Entry>) {
    this.version = version;
    this.entries = entries;
  }

  static fromMap(map: Map<string, any>): DirManifest {
    const version = map.get("version") as u16;
    const entriesArray = map.get("entries") as Array<any>;
    
    if (version != 1) {
      throw new Error("Unsupported manifest version");
    }
    
    if (!entriesArray || entriesArray.length > NGFS_MAX_DIR_ENTRIES) {
      throw new Error("Invalid entries array");
    }
    
    const entries = new Array<Entry>();
    for (let i = 0; i < entriesArray.length; i++) {
      const entryMap = entriesArray[i] as Map<string, any>;
      entries.push(Entry.fromMap(entryMap));
    }
    
    return new DirManifest(version, entries);
  }

  validate(): boolean {
    // Check entry count
    if (this.entries.length > NGFS_MAX_DIR_ENTRIES) {
      return false;
    }
    
    // Check canonical ordering
    for (let i = 1; i < this.entries.length; i++) {
      const prev = this.entries[i - 1];
      const curr = this.entries[i];
      
      // First sort by kind (ascending)
      if (prev.kind > curr.kind) {
        return false;
      }
      
      // If kinds are equal, sort by name (ascending)
      if (prev.kind === curr.kind && prev.name > curr.name) {
        return false;
      }
    }
    
    return true;
  }
}

/**
 * NGFS File Manifest structure
 */
class FileManifest {
  version: u16;
  chunks: Array<ChunkInfo>;
  totalSize: u64;
  algorithm: string;

  constructor(version: u16, chunks: Array<ChunkInfo>, totalSize: u64, algorithm: string) {
    this.version = version;
    this.chunks = chunks;
    this.totalSize = totalSize;
    this.algorithm = algorithm;
  }

  static fromMap(map: Map<string, any>): FileManifest {
    const version = map.get("version") as u16;
    const chunksArray = map.get("chunks") as Array<any>;
    const totalSize = map.get("total_size") as u64;
    const algorithm = map.get("algorithm") as string;
    
    if (version != 1) {
      throw new Error("Unsupported manifest version");
    }
    
    if (!chunksArray || chunksArray.length > NGFS_MAX_FILE_CHUNKS) {
      throw new Error("Invalid chunks array");
    }
    
    if (algorithm != "blake3") {
      throw new Error("Unsupported algorithm");
    }
    
    const chunks = new Array<ChunkInfo>();
    for (let i = 0; i < chunksArray.length; i++) {
      const chunkMap = chunksArray[i] as Map<string, any>;
      chunks.push(ChunkInfo.fromMap(chunkMap));
    }
    
    return new FileManifest(version, chunks, totalSize, algorithm);
  }

  validate(): boolean {
    // Check chunk count
    if (this.chunks.length > NGFS_MAX_FILE_CHUNKS) {
      return false;
    }
    
    // Check total size
    let computedSize: u64 = 0;
    for (let i = 0; i < this.chunks.length; i++) {
      computedSize += this.chunks[i].length;
    }
    
    if (computedSize !== this.totalSize) {
      return false;
    }
    
    // Check chunk ordering (should be sorted by CID)
    for (let i = 1; i < this.chunks.length; i++) {
      const prev = this.chunks[i - 1];
      const curr = this.chunks[i];
      
      // Compare CIDs byte by byte
      for (let j = 0; j < 32; j++) {
        if (prev.cid.blake3Hash[j] < curr.cid.blake3Hash[j]) break;
        if (prev.cid.blake3Hash[j] > curr.cid.blake3Hash[j]) {
          return false;
        }
      }
    }
    
    return true;
  }
}

/**
 * NGFS Proof structure (v1 skeleton)
 */
class Proof {
  nodeKind: u8;
  nodeCid: ContentID;
  path: Array<any>; // Simplified for v1

  constructor(nodeKind: u8, nodeCid: ContentID, path: Array<any>) {
    this.nodeKind = nodeKind;
    this.nodeCid = nodeCid;
    this.path = path;
  }

  static fromMap(map: Map<string, any>): Proof {
    const nodeKind = map.get("node_kind") as u8;
    const nodeCidMap = map.get("node_cid") as Map<string, any>;
    const path = map.get("path") as Array<any>;
    
    if (nodeKind > 2) {
      throw new Error("Invalid node kind");
    }
    
    const nodeCid = ContentID.fromMap(nodeCidMap);
    return new Proof(nodeKind, nodeCid, path);
  }

  validate(): boolean {
    // Basic validation for v1
    return this.nodeKind <= 2 && this.path.length > 0;
  }
}

/**
 * Parse NGFS manifest from CBOR data
 */
function parseManifest(cborData: Uint8Array): any {
  try {
    const decoder = new SimpleCBORDecoder(cborData);
    const decoded = decoder.decode();
    
    if (decoded instanceof Map) {
      const map = decoded as Map<string, any>;
      
      // Check if it's a directory manifest
      if (map.has("entries")) {
        return DirManifest.fromMap(map);
      }
      
      // Check if it's a file manifest
      if (map.has("chunks")) {
        return FileManifest.fromMap(map);
      }
    }
    
    throw new Error("Unknown manifest type");
    
  } catch (error) {
    throw new Error(`Failed to parse manifest: ${error.message}`);
  }
}

/**
 * Parse NGFS proof from CBOR data
 */
function parseProof(cborData: Uint8Array): Proof {
  try {
    const decoder = new SimpleCBORDecoder(cborData);
    const decoded = decoder.decode();
    
    if (decoded instanceof Map) {
      return Proof.fromMap(decoded as Map<string, any>);
    }
    
    throw new Error("Invalid proof format");
    
  } catch (error) {
    throw new Error(`Failed to parse proof: ${error.message}`);
  }
}

/**
 * Validate manifest and proof combination
 */
function validateManifestAndProof(manifestData: Uint8Array, proofData: Uint8Array): boolean {
  try {
    // Parse manifest
    const manifest = parseManifest(manifestData);
    
    // Parse proof
    const proof = parseProof(proofData);
    
    // Basic validation
    if (!manifest.validate() || !proof.validate()) {
      return false;
    }
    
    // For v1, we'll do basic structure validation
    // In v2, this would include proper Merkle tree verification
    
    return true;
    
  } catch (error) {
    return false;
  }
}

/**
 * Main skill function
 */
export function main(): void {
  console.log("NGFS Manifest & Proof Parser Skill");
  console.log("===================================");
  
  // This would normally receive data from the skill runtime
  // For demonstration, we'll create sample data
  
  console.log("Skill loaded successfully");
  console.log("Ready to parse NGFS manifests and proofs");
}

/**
 * Parse manifest data (called by skill runtime)
 */
export function parseManifestData(data: Uint8Array): string {
  try {
    const manifest = parseManifest(data);
    
    if (manifest instanceof DirManifest) {
      const dirManifest = manifest as DirManifest;
      return `Directory manifest: ${dirManifest.entries.length} entries, version ${dirManifest.version}`;
    } else if (manifest instanceof FileManifest) {
      const fileManifest = manifest as FileManifest;
      return `File manifest: ${fileManifest.chunks.length} chunks, ${fileManifest.totalSize} bytes, ${fileManifest.algorithm}`;
    } else {
      return "Unknown manifest type";
    }
    
  } catch (error) {
    return `Parse error: ${error.message}`;
  }
}

/**
 * Validate proof (called by skill runtime)
 */
export function validateProof(manifestData: Uint8Array, proofData: Uint8Array): string {
  try {
    const isValid = validateManifestAndProof(manifestData, proofData);
    
    if (isValid) {
      // Emit "proof.ok" preview event
      console.log("proof.ok");
      return "Proof validation successful";
    } else {
      return "Proof validation failed";
    }
    
  } catch (error) {
    return `Validation error: ${error.message}`;
  }
}

/**
 * Get skill metadata
 */
export function getSkillInfo(): string {
  return JSON.stringify({
    name: "NGFS Manifest Parser",
    version: "1.0.0",
    description: "Parses and validates NGFS manifests and proofs",
    capabilities: ["manifest_parsing", "proof_validation"],
    ngfs_version: "v1"
  });
}
