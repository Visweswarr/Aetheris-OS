#!/usr/bin/env node
/**
 * NGFS Manifest Validation Tool
 * 
 * This tool validates NGFS manifests using zod schema validation,
 * checks name normalization, and verifies canonical ordering.
 */

import { z } from 'zod';
import * as cbor from 'cbor';
import * as fs from 'fs';
import * as path from 'path';

// NGFS constants
const NGFS_MAX_NAME_LENGTH = 255;
const NGFS_MAX_DIR_ENTRIES = 65_535;
const NGFS_MAX_FILE_CHUNKS = 4_096;
const NGFS_MAX_FILE_SIZE = 1n << 40n; // 1 TiB

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

// Zod schemas for validation
const ContentIdSchema = z.object({
  blake3_hash: z.instanceof(Uint8Array).length(32),
  ipfs_multihash: z.instanceof(Uint8Array).optional(),
  content_type: z.number().int().min(0).max(5)
});

const EntrySchema = z.object({
  name: z.string().min(1).max(NGFS_MAX_NAME_LENGTH),
  kind: z.number().int().min(0).max(2),
  cid: ContentIdSchema,
  size: z.number().int().positive().optional(),
  mode: z.number().int().min(0).max(0o777).optional(),
  xattrs: z.record(z.string(), z.instanceof(Uint8Array)).optional()
});

const ChunkInfoSchema = z.object({
  cid: ContentIdSchema,
  length: z.number().int().positive().max(0xFFFFFFFF)
});

const FileManifestSchema = z.object({
  version: z.literal(1),
  chunks: z.array(ChunkInfoSchema).max(NGFS_MAX_FILE_CHUNKS),
  total_size: z.number().int().positive().max(Number(NGFS_MAX_FILE_SIZE)),
  algorithm: z.literal('blake3')
});

const DirManifestSchema = z.object({
  version: z.literal(1),
  entries: z.array(EntrySchema).max(NGFS_MAX_DIR_ENTRIES)
});

const ManifestSchema = z.union([FileManifestSchema, DirManifestSchema]);

// Validation result type
type ValidationResult = {
  ok: true;
  manifest_type: 'file' | 'directory';
  entry_count: number;
  size: number;
  cid: string;
} | {
  ok: false;
  error: string;
  details?: any;
};

/**
 * Normalize filename to NFC form
 */
function normalizeName(name: string): string {
  return name.normalize('NFC');
}

/**
 * Validate entry name according to NGFS constraints
 */
function validateEntryName(name: string): { valid: boolean; error?: string } {
  if (!name || name.length > NGFS_MAX_NAME_LENGTH) {
    return { 
      valid: false, 
      error: `Name length ${name.length} exceeds maximum ${NGFS_MAX_NAME_LENGTH}` 
    };
  }
  
  // Check for forbidden characters
  if (name.includes('\x00')) {
    return { valid: false, error: 'Name contains NUL character' };
  }
  
  if (name.includes('/')) {
    return { valid: false, error: 'Name contains forward slash' };
  }
  
  // Check for reserved names
  if (name === '.' || name === '..') {
    return { valid: false, error: `Name '${name}' is reserved` };
  }
  
  // Check for control characters
  for (const char of name) {
    if (char.charCodeAt(0) < 0x20) {
      return { valid: false, error: `Name contains control character: ${char.charCodeAt(0)}` };
    }
  }
  
  return { valid: true };
}

/**
 * Check if entries are in canonical order
 */
function checkCanonicalOrder(entries: any[]): { valid: boolean; error?: string } {
  for (let i = 1; i < entries.length; i++) {
    const prev = entries[i - 1];
    const curr = entries[i];
    
    // First sort by kind (ascending)
    if (prev.kind > curr.kind) {
      return { 
        valid: false, 
        error: `Entry order violation: ${prev.name} (kind ${prev.kind}) should come after ${curr.name} (kind ${curr.kind})` 
      };
    }
    
    // If kinds are equal, sort by name (ascending)
    if (prev.kind === curr.kind && prev.name > curr.name) {
      return { 
        valid: false, 
        error: `Entry order violation: ${prev.name} should come after ${curr.name} within same kind ${curr.kind}` 
      };
    }
  }
  
  return { valid: true };
}

/**
 * Check if chunks are in canonical order (sorted by CID)
 */
function checkChunkOrder(chunks: any[]): { valid: boolean; error?: string } {
  for (let i = 1; i < chunks.length; i++) {
    const prev = chunks[i - 1];
    const curr = chunks[i];
    
    // Compare CIDs byte by byte
    const prevCid = prev.cid.blake3_hash;
    const currCid = curr.cid.blake3_hash;
    
    for (let j = 0; j < 32; j++) {
      if (prevCid[j] < currCid[j]) break;
      if (prevCid[j] > currCid[j]) {
        return { 
          valid: false, 
          error: `Chunk order violation: chunk ${i-1} CID should come after chunk ${i} CID` 
        };
      }
    }
  }
  
  return { valid: true };
}

/**
 * Compute content identifier from CBOR bytes
 */
function computeCID(cborBytes: Uint8Array, contentType: number): string {
  // For now, we'll use a simple hash function
  // In a real implementation, this would use Blake3
  let hash = 0;
  for (let i = 0; i < cborBytes.length; i++) {
    hash = ((hash << 5) - hash + cborBytes[i]) & 0xFFFFFFFF;
  }
  return hash.toString(16).padStart(8, '0');
}

/**
 * Validate a manifest from CBOR bytes
 */
function validateManifest(cborBytes: Uint8Array): ValidationResult {
  try {
    // Decode CBOR
    const decoded = cbor.decode(cborBytes);
    
    // Validate against schema
    const manifest = ManifestSchema.parse(decoded);
    
    // Determine manifest type
    if ('chunks' in manifest) {
      // File manifest
      const fileManifest = manifest as z.infer<typeof FileManifestSchema>;
      
      // Check chunk order
      const orderCheck = checkChunkOrder(fileManifest.chunks);
      if (!orderCheck.valid) {
        return { ok: false, error: orderCheck.error! };
      }
      
      // Validate total size
      const computedSize = fileManifest.chunks.reduce((sum, chunk) => sum + chunk.length, 0);
      if (computedSize !== fileManifest.total_size) {
        return { 
          ok: false, 
          error: `Total size mismatch: computed ${computedSize}, manifest ${fileManifest.total_size}` 
        };
      }
      
      // Compute CID
      const cid = computeCID(cborBytes, CONTENT_TYPE_FILE_MANIFEST);
      
      return {
        ok: true,
        manifest_type: 'file',
        entry_count: fileManifest.chunks.length,
        size: fileManifest.total_size,
        cid
      };
      
    } else {
      // Directory manifest
      const dirManifest = manifest as z.infer<typeof DirManifestSchema>;
      
      // Check entry order
      const orderCheck = checkCanonicalOrder(dirManifest.entries);
      if (!orderCheck.valid) {
        return { ok: false, error: orderCheck.error! };
      }
      
      // Validate entry names
      for (const entry of dirManifest.entries) {
        const nameCheck = validateEntryName(entry.name);
        if (!nameCheck.valid) {
          return { ok: false, error: `Invalid entry name '${entry.name}': ${nameCheck.error}` };
        }
        
        // Check if name is normalized
        const normalized = normalizeName(entry.name);
        if (normalized !== entry.name) {
          return { 
            ok: false, 
            error: `Entry name '${entry.name}' is not normalized (should be '${normalized}')` 
          };
        }
      }
      
      // Compute CID
      const cid = computeCID(cborBytes, CONTENT_TYPE_DIRECTORY);
      
      return {
        ok: true,
        manifest_type: 'directory',
        entry_count: dirManifest.entries.length,
        size: cborBytes.length,
        cid
      };
    }
    
  } catch (error) {
    if (error instanceof z.ZodError) {
      return { 
        ok: false, 
        error: 'Schema validation failed', 
        details: error.errors 
      };
    }
    
    if (error instanceof Error) {
      return { 
        ok: false, 
        error: `CBOR decode failed: ${error.message}` 
      };
    }
    
    return { 
      ok: false, 
      error: 'Unknown validation error' 
    };
  }
}

/**
 * Generate a test manifest for validation
 */
function generateTestManifest(type: 'file' | 'directory'): Uint8Array {
  if (type === 'file') {
    // Generate a simple file manifest
    const manifest = {
      version: 1,
      chunks: [
        {
          cid: {
            blake3_hash: new Uint8Array(32).fill(1),
            content_type: CONTENT_TYPE_RAW
          },
          length: 100
        },
        {
          cid: {
            blake3_hash: new Uint8Array(32).fill(2),
            content_type: CONTENT_TYPE_RAW
          },
          length: 200
        }
      ],
      total_size: 300,
      algorithm: 'blake3'
    };
    
    return cbor.encode(manifest);
    
  } else {
    // Generate a simple directory manifest
    const manifest = {
      version: 1,
      entries: [
        {
          name: 'a_dir',
          kind: ENTRY_KIND_DIRECTORY,
          cid: {
            blake3_hash: new Uint8Array(32).fill(1),
            content_type: CONTENT_TYPE_DIRECTORY
          }
        },
        {
          name: 'b_file',
          kind: ENTRY_KIND_FILE,
          cid: {
            blake3_hash: new Uint8Array(32).fill(2),
            content_type: CONTENT_TYPE_RAW
          },
          size: 100,
          mode: 0o644
        }
      ]
    };
    
    return cbor.encode(manifest);
  }
}

/**
 * Main function
 */
function main(): void {
  const args = process.argv.slice(2);
  
  if (args.length === 0) {
    console.log('NGFS Manifest Validation Tool');
    console.log('');
    console.log('Usage:');
    console.log('  node validate_manifest.js <manifest.cbor>');
    console.log('  node validate_manifest.js --test');
    console.log('');
    console.log('Options:');
    console.log('  --test    Generate and validate test manifests');
    console.log('');
    process.exit(1);
  }
  
  if (args[0] === '--test') {
    console.log('Generating and validating test manifests...\n');
    
    // Test file manifest
    console.log('Testing file manifest:');
    const fileManifest = generateTestManifest('file');
    const fileResult = validateManifest(fileManifest);
    
    if (fileResult.ok) {
      console.log(`  ✓ File manifest valid: ${fileResult.entry_count} chunks, ${fileResult.size} bytes, CID: ${fileResult.cid}`);
    } else {
      console.log(`  ✗ File manifest invalid: ${fileResult.error}`);
      if (fileResult.details) {
        console.log('    Details:', JSON.stringify(fileResult.details, null, 2));
      }
    }
    
    console.log('');
    
    // Test directory manifest
    console.log('Testing directory manifest:');
    const dirManifest = generateTestManifest('directory');
    const dirResult = validateManifest(dirManifest);
    
    if (dirResult.ok) {
      console.log(`  ✓ Directory manifest valid: ${dirResult.entry_count} entries, ${dirResult.size} bytes, CID: ${dirResult.cid}`);
    } else {
      console.log(`  ✗ Directory manifest invalid: ${dirResult.error}`);
      if (dirResult.details) {
        console.log('    Details:', JSON.stringify(dirResult.details, null, 2));
      }
    }
    
    return;
  }
  
  // Validate manifest file
  const manifestPath = args[0];
  
  if (!fs.existsSync(manifestPath)) {
    console.error(`Error: Manifest file '${manifestPath}' does not exist`);
    process.exit(1);
  }
  
  try {
    const cborBytes = fs.readFileSync(manifestPath);
    const result = validateManifest(cborBytes);
    
    if (result.ok) {
      console.log('✓ Manifest validation successful');
      console.log(`  Type: ${result.manifest_type}`);
      console.log(`  Entries/Chunks: ${result.entry_count}`);
      console.log(`  Size: ${result.size} bytes`);
      console.log(`  CID: ${result.cid}`);
      process.exit(0);
    } else {
      console.error('✗ Manifest validation failed');
      console.error(`  Error: ${result.error}`);
      if (result.details) {
        console.error('  Details:', JSON.stringify(result.details, null, 2));
      }
      process.exit(1);
    }
    
  } catch (error) {
    console.error('Error reading manifest file:', error);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main();
}

export {
  validateManifest,
  validateEntryName,
  normalizeName,
  checkCanonicalOrder,
  checkChunkOrder,
  computeCID,
  generateTestManifest
};
