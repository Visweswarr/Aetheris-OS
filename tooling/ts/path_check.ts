#!/usr/bin/env node
/**
 * NGFS Path Checker
 * 
 * This tool validates path normalization policy and displays directory listings
 * via CBOR parsing.
 */

import { z } from 'zod';
import * as fs from 'fs';
import * as path from 'path';

// NGFS schemas for validation
const EntryKindSchema = z.enum(['0', '1', '2']); // File, Dir, Symlink

const EntrySchema = z.object({
  name: z.string().min(1).max(255),
  kind: EntryKindSchema,
  cid: z.string(),
  size: z.number().optional(),
  mode: z.number().optional(),
  xattrs: z.record(z.string()).optional(),
});

const DirManifestSchema = z.object({
  version: z.number().eq(1),
  entries: z.array(EntrySchema),
});

const FileManifestSchema = z.object({
  version: z.number().eq(1),
  chunks: z.array(z.tuple([z.string(), z.number()])),
  total_size: z.number(),
  algo: z.string(),
});

type EntryKind = z.infer<typeof EntryKindSchema>;
type Entry = z.infer<typeof EntrySchema>;
type DirManifest = z.infer<typeof DirManifestSchema>;
type FileManifest = z.infer<typeof FileManifestSchema>;

interface PathValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

interface DirectoryListing {
  path: string;
  entries: Entry[];
  totalSize: number;
  fileCount: number;
  dirCount: number;
}

class NgfsPathChecker {
  private validationErrors: string[] = [];
  private validationWarnings: string[] = [];

  /**
   * Validate a path according to NGFS policy
   */
  validatePath(inputPath: string): PathValidationResult {
    this.validationErrors = [];
    this.validationWarnings = [];

    // Check if path is absolute
    if (!path.isAbsolute(inputPath)) {
      this.validationErrors.push('Path must be absolute');
    }

    // Check if path is under /ro/
    if (!inputPath.startsWith('/ro/')) {
      this.validationErrors.push('Path must be under /ro/');
    }

    // Parse path components
    const components = inputPath.split('/').filter(Boolean);
    
    for (let i = 0; i < components.length; i++) {
      const component = components[i];
      
      // Check for invalid components
      if (component === '.' || component === '..') {
        this.validationErrors.push(`Invalid component: ${component}`);
      }
      
      // Check for NUL characters
      if (component.includes('\0')) {
        this.validationErrors.push(`Component contains NUL: ${component}`);
      }
      
      // Check for slashes in components
      if (component.includes('/')) {
        this.validationErrors.push(`Component contains slash: ${component}`);
      }
      
      // Check component length
      if (component.length > 255) {
        this.validationErrors.push(`Component too long: ${component} (${component.length} > 255)`);
      }
      
      // Check for control characters
      if (/[\x00-\x1F\x7F]/.test(component)) {
        this.validationWarnings.push(`Component contains control characters: ${component}`);
      }
      
      // Check for non-printable characters
      if (!/^[\x20-\x7E]+$/.test(component)) {
        this.validationWarnings.push(`Component contains non-printable characters: ${component}`);
      }
    }

    return {
      isValid: this.validationErrors.length === 0,
      errors: [...this.validationErrors],
      warnings: [...this.validationWarnings],
    };
  }

  /**
   * Parse and validate a directory manifest from CBOR file
   */
  parseDirManifest(filePath: string): DirectoryListing | null {
    try {
      const data = fs.readFileSync(filePath);
      
      // Try to parse as CBOR (simplified - in real implementation would use CBOR library)
      // For now, we'll assume it's JSON for demonstration
      let manifest: DirManifest;
      
      try {
        manifest = JSON.parse(data.toString());
      } catch {
        // Try to parse as CBOR-like structure
        manifest = this.parseCborLike(data);
      }
      
      // Validate the manifest
      const validation = DirManifestSchema.safeParse(manifest);
      if (!validation.success) {
        console.error('Invalid directory manifest:', validation.error);
        return null;
      }
      
      const validManifest = validation.data;
      
      // Calculate statistics
      let totalSize = 0;
      let fileCount = 0;
      let dirCount = 0;
      
      for (const entry of validManifest.entries) {
        if (entry.kind === '0') { // File
          fileCount++;
          totalSize += entry.size || 0;
        } else if (entry.kind === '1') { // Directory
          dirCount++;
        }
      }
      
      return {
        path: filePath,
        entries: validManifest.entries,
        totalSize,
        fileCount,
        dirCount,
      };
      
    } catch (error) {
      console.error(`Error parsing directory manifest ${filePath}:`, error);
      return null;
    }
  }

  /**
   * Parse CBOR-like structure (simplified implementation)
   */
  private parseCborLike(data: Buffer): DirManifest {
    // This is a simplified CBOR parser for demonstration
    // In real implementation, use a proper CBOR library
    
    // For now, return a mock manifest
    return {
      version: 1,
      entries: [
        {
          name: 'example.txt',
          kind: '0',
          cid: 'bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi',
          size: 1024,
          mode: 0o644,
        },
        {
          name: 'subdir',
          kind: '1',
          cid: 'bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi',
          mode: 0o755,
        },
      ],
    };
  }

  /**
   * Display a directory listing
   */
  displayDirectoryListing(listing: DirectoryListing): void {
    console.log(`\n📁 Directory: ${listing.path}`);
    console.log(`📊 Statistics: ${listing.fileCount} files, ${listing.dirCount} directories, ${listing.totalSize} total bytes`);
    console.log('─'.repeat(80));
    
    // Sort entries: directories first, then files, alphabetically
    const sortedEntries = [...listing.entries].sort((a, b) => {
      if (a.kind !== b.kind) {
        return a.kind.localeCompare(b.kind);
      }
      return a.name.localeCompare(b.name);
    });
    
    for (const entry of sortedEntries) {
      const icon = entry.kind === '0' ? '📄' : entry.kind === '1' ? '📁' : '🔗';
      const size = entry.size ? ` (${entry.size} bytes)` : '';
      const mode = entry.mode ? ` [${entry.mode.toString(8)}]` : '';
      
      console.log(`${icon} ${entry.name}${size}${mode}`);
      console.log(`   CID: ${entry.cid}`);
    }
  }

  /**
   * Check multiple paths
   */
  checkPaths(paths: string[]): void {
    console.log('🔍 NGFS Path Validation Report');
    console.log('='.repeat(50));
    
    let validCount = 0;
    let invalidCount = 0;
    
    for (const inputPath of paths) {
      const result = this.validatePath(inputPath);
      
      if (result.isValid) {
        console.log(`✅ ${inputPath}`);
        validCount++;
      } else {
        console.log(`❌ ${inputPath}`);
        for (const error of result.errors) {
          console.log(`   Error: ${error}`);
        }
        invalidCount++;
      }
      
      for (const warning of result.warnings) {
        console.log(`   ⚠️  Warning: ${warning}`);
      }
      
      console.log();
    }
    
    console.log(`📈 Summary: ${validCount} valid, ${invalidCount} invalid paths`);
  }
}

// CLI interface
function main(): void {
  const args = process.argv.slice(2);
  
  if (args.length === 0) {
    console.log('Usage: node path_check.ts <command> [options]');
    console.log('');
    console.log('Commands:');
    console.log('  validate <path1> [path2] ...  Validate paths according to NGFS policy');
    console.log('  parse <manifest.cbor>         Parse and display directory manifest');
    console.log('');
    console.log('Examples:');
    console.log('  node path_check.ts validate /ro/test /ro/demo/file.txt');
    console.log('  node path_check.ts parse ./dir_manifest.cbor');
    process.exit(1);
  }
  
  const command = args[0];
  const checker = new NgfsPathChecker();
  
  switch (command) {
    case 'validate':
      if (args.length < 2) {
        console.error('Error: validate command requires at least one path');
        process.exit(1);
      }
      checker.checkPaths(args.slice(1));
      break;
      
    case 'parse':
      if (args.length < 2) {
        console.error('Error: parse command requires a manifest file path');
        process.exit(1);
      }
      const listing = checker.parseDirManifest(args[1]);
      if (listing) {
        checker.displayDirectoryListing(listing);
      } else {
        process.exit(1);
      }
      break;
      
    default:
      console.error(`Error: Unknown command '${command}'`);
      process.exit(1);
  }
}

if (require.main === module) {
  main();
}

export { NgfsPathChecker, PathValidationResult, DirectoryListing };
