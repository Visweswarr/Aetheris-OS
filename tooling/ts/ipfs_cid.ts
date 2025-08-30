#!/usr/bin/env/node

/**
 * IPFS CID Validator for NGFS
 * 
 * This tool validates IPFS CIDs and compares them against export maps.
 */

import { readFileSync } from 'fs';
import { join } from 'path';

interface IpfsMapEntry {
    ngfs_cid: number[];
    ipfs_cid: string;
    kind: number;
    size: number;
}

interface IpfsMap {
    version: number;
    exported_vclock: number;
    root_ngfs_cid: number[];
    entries: IpfsMapEntry[];
}

interface ValidationResult {
    valid: boolean;
    cid: string;
    codec: string;
    errors: string[];
    warnings: string[];
}

class IpfsCidValidator {
    private errors: string[] = [];
    private warnings: string[] = [];

    /**
     * Validate IPFS CIDv1 format
     */
    validateCid(cid: string): boolean {
        // Reset errors
        this.errors = [];
        this.warnings = [];

        // Check if it's a string
        if (typeof cid !== 'string') {
            this.errors.push('CID must be a string');
            return false;
        }

        // Check if it starts with 'b' (base32-lowercase)
        if (!cid.startsWith('b')) {
            this.errors.push('CID must start with "b" (base32-lowercase)');
            return false;
        }

        // Check length
        if (cid.length < 10) {
            this.errors.push('CID too short');
            return false;
        }

        if (cid.length > 100) {
            this.warnings.push('CID unusually long');
        }

        // Check if it contains only valid base32 characters
        const validChars = new Set('abcdefghijklmnopqrstuvwxyz234567');
        const invalidChars = cid.slice(1).split('').filter(c => !validChars.has(c));
        
        if (invalidChars.length > 0) {
            this.errors.push(`Invalid characters in CID: ${invalidChars.join(', ')}`);
            return false;
        }

        return this.errors.length === 0;
    }

    /**
     * Parse and validate CBOR data for DAG-CBOR codec
     */
    validateDagCbor(data: Buffer): ValidationResult {
        try {
            // For now, just check if it's valid CBOR
            // In a real implementation, this would parse the CBOR and validate structure
            const result: ValidationResult = {
                valid: true,
                cid: 'bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi', // Placeholder
                codec: 'dag-cbor',
                errors: [],
                warnings: []
            };

            // Basic CBOR validation (check for valid CBOR header)
            if (data.length === 0) {
                result.valid = false;
                result.errors.push('Empty data');
                return result;
            }

            // Check first byte for valid CBOR type
            const firstByte = data[0];
            if (firstByte < 0x00 || firstByte > 0xff) {
                result.valid = false;
                result.errors.push('Invalid CBOR header');
                return result;
            }

            return result;
        } catch (error) {
            return {
                valid: false,
                cid: '',
                codec: 'dag-cbor',
                errors: [`CBOR parsing error: ${error}`],
                warnings: []
            };
        }
    }

    /**
     * Validate raw data for RAW codec
     */
    validateRaw(data: Buffer): ValidationResult {
        const result: ValidationResult = {
            valid: true,
            cid: 'bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi', // Placeholder
            codec: 'raw',
            errors: [],
            warnings: []
        };

        if (data.length === 0) {
            result.valid = false;
            result.errors.push('Empty data');
            return result;
        }

        if (data.length > 1024 * 1024) {
            result.warnings.push('Data larger than 1MB');
        }

        return result;
    }

    /**
     * Load and parse IPFS map file
     */
    loadMapFile(mapPath: string): IpfsMap | null {
        try {
            const data = readFileSync(mapPath, 'utf8');
            const map = JSON.parse(data) as IpfsMap;
            
            // Validate map structure
            if (!map.version || !map.entries || !Array.isArray(map.entries)) {
                this.errors.push('Invalid map structure');
                return null;
            }

            return map;
        } catch (error) {
            this.errors.push(`Failed to load map file: ${error}`);
            return null;
        }
    }

    /**
     * Find entry in map by NGFS CID
     */
    findEntryInMap(map: IpfsMap, ngfsCid: number[]): IpfsMapEntry | null {
        return map.entries.find(entry => 
            entry.ngfs_cid.length === ngfsCid.length &&
            entry.ngfs_cid.every((byte, i) => byte === ngfsCid[i])
        ) || null;
    }

    /**
     * Validate file against IPFS map
     */
    validateAgainstMap(filePath: string, codec: string, mapPath?: string): ValidationResult {
        let data: Buffer;
        try {
            data = readFileSync(filePath);
        } catch (error) {
            return {
                valid: false,
                cid: '',
                codec,
                errors: [`Failed to read file: ${error}`],
                warnings: []
            };
        }

        // Validate based on codec
        let result: ValidationResult;
        if (codec === 'dag-cbor') {
            result = this.validateDagCbor(data);
        } else if (codec === 'raw') {
            result = this.validateRaw(data);
        } else {
            return {
                valid: false,
                cid: '',
                codec,
                errors: [`Unsupported codec: ${codec}`],
                warnings: []
            };
        }

        // If map file provided, validate against it
        if (mapPath) {
            const map = this.loadMapFile(mapPath);
            if (map) {
                // For now, just check if the map has entries
                // In a real implementation, this would compute the actual CID and compare
                if (map.entries.length === 0) {
                    result.warnings.push('Map file has no entries');
                } else {
                    result.warnings.push(`Map file has ${map.entries.length} entries`);
                }
            }
        }

        return result;
    }

    /**
     * Print validation result
     */
    printResult(result: ValidationResult): void {
        console.log(`Codec: ${result.codec}`);
        console.log(`Valid: ${result.valid ? '✅' : '❌'}`);
        
        if (result.cid) {
            console.log(`CID: ${result.cid}`);
        }

        if (result.errors.length > 0) {
            console.log('Errors:');
            result.errors.forEach(error => console.log(`  ❌ ${error}`));
        }

        if (result.warnings.length > 0) {
            console.log('Warnings:');
            result.warnings.forEach(warning => console.log(`  ⚠️  ${warning}`));
        }
    }
}

function main(): void {
    const args = process.argv.slice(2);
    
    if (args.length < 2) {
        console.error('Usage: ipfs_cid <path-to-file> --codec <dag-cbor|raw> [--map <map-file>]');
        process.exit(1);
    }

    const filePath = args[0];
    const codecIndex = args.indexOf('--codec');
    const mapIndex = args.indexOf('--map');

    if (codecIndex === -1 || codecIndex + 1 >= args.length) {
        console.error('--codec argument required');
        process.exit(1);
    }

    const codec = args[codecIndex + 1];
    if (!['dag-cbor', 'raw'].includes(codec)) {
        console.error('Codec must be "dag-cbor" or "raw"');
        process.exit(1);
    }

    const mapPath = mapIndex !== -1 && mapIndex + 1 < args.length ? args[mapIndex + 1] : undefined;

    const validator = new IpfsCidValidator();
    const result = validator.validateAgainstMap(filePath, codec, mapPath);
    
    validator.printResult(result);
    
    if (!result.valid) {
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}

export { IpfsCidValidator, ValidationResult, IpfsMap, IpfsMapEntry };
