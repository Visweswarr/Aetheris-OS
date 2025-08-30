#!/usr/bin/env node
/**
 * NGFS Envelope Header Validator
 * 
 * This tool validates NGFS encryption headers using CBOR decoding
 * and zod schema validation.
 */

import { z } from 'zod';
import * as cbor from 'cbor';

// NGFS encryption header schema
const EncryptionAlgSchema = z.enum(['XChaCha20Poly1305', 'ChaCha20Poly1305', 'Aes256Gcm']);

const EncHeaderV1Schema = z.object({
  key_id: z.string(),
  algorithm: EncryptionAlgSchema,
  nonce: z.instanceof(Uint8Array).refine(
    (data) => data.length === 24,
    'Nonce must be exactly 24 bytes'
  ),
  tag: z.instanceof(Uint8Array).refine(
    (data) => data.length === 16,
    'Tag must be exactly 16 bytes'
  ),
  aad: z.instanceof(Uint8Array).optional(),
});

// Associated data schema
const AssociatedDataSchema = z.object({
  schema_hash: z.instanceof(Uint8Array).refine(
    (data) => data.length === 32,
    'Schema hash must be exactly 32 bytes'
  ),
  key_id: z.string(),
  algorithm: EncryptionAlgSchema,
  chunk_len: z.number().int().positive().max(256 * 1024),
  metadata: z.record(z.string(), z.string()),
});

// Validation result type
type ValidationResult = 
  | { ok: true; header: any; associatedData?: any }
  | { ok: false; error: string };

/**
 * Validate NGFS encryption header from CBOR bytes
 */
function validateEncHeader(cborBytes: Uint8Array): ValidationResult {
  try {
    // Decode CBOR
    const decoded = cbor.decode(cborBytes);
    
    // Validate header structure
    const header = EncHeaderV1Schema.parse(decoded);
    
    // Validate associated data if present
    let associatedData: any = undefined;
    if (header.aad) {
      try {
        const adDecoded = cbor.decode(header.aad);
        associatedData = AssociatedDataSchema.parse(adDecoded);
      } catch (adError) {
        return {
          ok: false,
          error: `Invalid associated data: ${adError instanceof Error ? adError.message : 'Unknown error'}`
        };
      }
    }
    
    return {
      ok: true,
      header,
      associatedData
    };
    
  } catch (error) {
    return {
      ok: false,
      error: error instanceof Error ? error.message : 'Unknown validation error'
    };
  }
}

/**
 * Validate header from file
 */
async function validateHeaderFile(filePath: string): Promise<ValidationResult> {
  try {
    const fs = await import('fs/promises');
    const data = await fs.readFile(filePath);
    return validateEncHeader(new Uint8Array(data));
  } catch (error) {
    return {
      ok: false,
      error: `Failed to read file: ${error instanceof Error ? error.message : 'Unknown error'}`
    };
  }
}

/**
 * Generate test header for validation
 */
function generateTestHeader(): Uint8Array {
  const testHeader = {
    key_id: "test-key-123",
    algorithm: "XChaCha20Poly1305",
    nonce: new Uint8Array(24).fill(1),
    tag: new Uint8Array(16).fill(2),
    aad: cbor.encode({
      schema_hash: new Uint8Array(32).fill(3),
      key_id: "test-key-123",
      algorithm: "XChaCha20Poly1305",
      chunk_len: 1024,
      metadata: {
        "test": "value",
        "version": "1.0"
      }
    })
  };
  
  return cbor.encode(testHeader);
}

/**
 * Main function
 */
async function main() {
  const args = process.argv.slice(2);
  
  if (args.length === 0) {
    console.log("NGFS Envelope Header Validator");
    console.log("Usage: validate_envelope.ts <header.cbor>");
    console.log("   or: validate_envelope.ts --test");
    console.log("");
    console.log("Options:");
    console.log("  --test     Generate and validate a test header");
    console.log("  <file>     Validate header from CBOR file");
    return;
  }
  
  if (args[0] === '--test') {
    console.log("Generating test header...");
    const testHeader = generateTestHeader();
    
    console.log("Validating test header...");
    const result = validateEncHeader(testHeader);
    
    if (result.ok) {
      console.log("✓ Test header validation passed");
      console.log("Header:", JSON.stringify(result.header, null, 2));
      if (result.associatedData) {
        console.log("Associated Data:", JSON.stringify(result.associatedData, null, 2));
      }
    } else {
      console.log("✗ Test header validation failed:", result.error);
      process.exit(1);
    }
    
    return;
  }
  
  // Validate header from file
  const filePath = args[0];
  console.log(`Validating header from: ${filePath}`);
  
  const result = await validateHeaderFile(filePath);
  
  if (result.ok) {
    console.log("✓ Header validation passed");
    console.log("Key ID:", result.header.key_id);
    console.log("Algorithm:", result.header.algorithm);
    console.log("Nonce length:", result.header.nonce.length);
    console.log("Tag length:", result.header.tag.length);
    
    if (result.associatedData) {
      console.log("Associated Data:");
      console.log("  Schema hash length:", result.associatedData.schema_hash.length);
      console.log("  Chunk length:", result.associatedData.chunk_len);
      console.log("  Metadata keys:", Object.keys(result.associatedData.metadata).length);
    }
  } else {
    console.log("✗ Header validation failed:", result.error);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main().catch((error) => {
    console.error("Error:", error);
    process.exit(1);
  });
}

export { validateEncHeader, EncHeaderV1Schema, AssociatedDataSchema };
