#!/usr/bin/env node
/**
 * Design Tokens Generator for Aetheris OS
 * Generates design tokens in multiple formats from YAML source
 */

import * as fs from 'fs';
import * as path from 'path';
import * as yaml from 'js-yaml';

// Types
interface DesignTokens {
  metadata: {
    name: string;
    version: string;
    description: string;
    lastModified: string;
  };
  colors: any;
  typography: any;
  spacing: any;
  radius: any;
  elevation: any;
  motion: any;
  breakpoints: any;
  zIndex: any;
  components: any;
  semantic: any;
}

// Configuration
const CONFIG = {
  sourceFile: 'design/Design-Tokens.yaml',
  outputDir: 'ui/tokens',
  formats: ['css', 'json', 'rs', 'ts'] as const,
};

// Utility functions
function ensureDir(dirPath: string): void {
  if (!fs.existsSync(dirPath)) {
    fs.mkdirSync(dirPath, { recursive: true });
  }
}

function flattenObject(obj: any, prefix = '', result: Record<string, any> = {}): Record<string, any> {
  for (const key in obj) {
    if (obj.hasOwnProperty(key)) {
      const newKey = prefix ? `${prefix}-${key}` : key;
      
      if (typeof obj[key] === 'object' && obj[key] !== null && !Array.isArray(obj[key])) {
        flattenObject(obj[key], newKey, result);
      } else {
        result[newKey] = obj[key];
      }
    }
  }
  return result;
}

function toKebabCase(str: string): string {
  return str.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase();
}

function toPascalCase(str: string): string {
  return str.replace(/(?:^|[-_])(\w)/g, (_, c) => c.toUpperCase());
}

function toSnakeCase(str: string): string {
  return str.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase();
}

// CSS Generator
function generateCSS(tokens: DesignTokens): string {
  const flattened = flattenObject(tokens);
  const cssVars: string[] = [];
  
  cssVars.push('/* Aetheris OS Design Tokens - Generated CSS Variables */');
  cssVars.push(`/* Generated from: ${CONFIG.sourceFile} */`);
  cssVars.push(`/* Version: ${tokens.metadata.version} */`);
  cssVars.push(`/* Last Modified: ${tokens.metadata.lastModified} */`);
  cssVars.push('');
  cssVars.push(':root {');
  
  // Generate CSS custom properties
  for (const [key, value] of Object.entries(flattened)) {
    if (typeof value === 'string' || typeof value === 'number') {
      const cssVar = `--${toKebabCase(key)}`;
      cssVars.push(`  ${cssVar}: ${value};`);
    }
  }
  
  cssVars.push('}');
  cssVars.push('');
  
  // Add dark mode variants
  cssVars.push('@media (prefers-color-scheme: dark) {');
  cssVars.push('  :root {');
  
  // Generate dark mode variants for colors
  for (const [key, value] of Object.entries(flattened)) {
    if (key.includes('light') && typeof value === 'string') {
      const darkKey = key.replace('light', 'dark');
      if (flattened[darkKey]) {
        const cssVar = `--${toKebabCase(key.replace('-light', ''))}`;
        cssVars.push(`    ${cssVar}: ${flattened[darkKey]};`);
      }
    }
  }
  
  cssVars.push('  }');
  cssVars.push('}');
  
  return cssVars.join('\n');
}

// JSON Generator
function generateJSON(tokens: DesignTokens): string {
  const flattened = flattenObject(tokens);
  return JSON.stringify({
    metadata: tokens.metadata,
    tokens: flattened,
    generated: new Date().toISOString(),
  }, null, 2);
}

// Rust Generator
function generateRust(tokens: DesignTokens): string {
  const flattened = flattenObject(tokens);
  const rustCode: string[] = [];
  
  rustCode.push('//! Aetheris OS Design Tokens - Generated Rust Constants');
  rustCode.push(`//! Generated from: ${CONFIG.sourceFile}`);
  rustCode.push(`//! Version: ${tokens.metadata.version}`);
  rustCode.push(`//! Last Modified: ${tokens.metadata.lastModified}`);
  rustCode.push('');
  rustCode.push('/// Design tokens for Aetheris OS');
  rustCode.push('pub mod tokens {');
  rustCode.push('');
  
  // Generate constants
  for (const [key, value] of Object.entries(flattened)) {
    if (typeof value === 'string' || typeof value === 'number') {
      const constName = toSnakeCase(key).replace(/-/g, '_').toUpperCase();
      const rustValue = typeof value === 'string' ? `"${value}"` : value;
      rustCode.push(`    /// ${key}`);
      rustCode.push(`    pub const ${constName}: &str = ${rustValue};`);
      rustCode.push('');
    }
  }
  
  rustCode.push('}');
  
  return rustCode.join('\n');
}

// TypeScript Generator
function generateTypeScript(tokens: DesignTokens): string {
  const flattened = flattenObject(tokens);
  const tsCode: string[] = [];
  
  tsCode.push('/**');
  tsCode.push(' * Aetheris OS Design Tokens - Generated TypeScript');
  tsCode.push(` * Generated from: ${CONFIG.sourceFile}`);
  tsCode.push(` * Version: ${tokens.metadata.version}`);
  tsCode.push(` * Last Modified: ${tokens.metadata.lastModified}`);
  tsCode.push(' */');
  tsCode.push('');
  
  // Generate type definitions
  tsCode.push('export interface DesignTokens {');
  for (const [key, value] of Object.entries(flattened)) {
    if (typeof value === 'string' || typeof value === 'number') {
      const tsKey = toKebabCase(key);
      const tsType = typeof value === 'string' ? 'string' : 'number';
      tsCode.push(`  "${tsKey}": ${tsType};`);
    }
  }
  tsCode.push('}');
  tsCode.push('');
  
  // Generate tokens object
  tsCode.push('export const tokens: DesignTokens = {');
  for (const [key, value] of Object.entries(flattened)) {
    if (typeof value === 'string' || typeof value === 'number') {
      const tsKey = toKebabCase(key);
      const tsValue = typeof value === 'string' ? `"${value}"` : value;
      tsCode.push(`  "${tsKey}": ${tsValue},`);
    }
  }
  tsCode.push('};');
  tsCode.push('');
  
  // Generate utility functions
  tsCode.push('/**');
  tsCode.push(' * Get a design token value');
  tsCode.push(' */');
  tsCode.push('export function getToken(key: keyof DesignTokens): string | number {');
  tsCode.push('  return tokens[key];');
  tsCode.push('}');
  tsCode.push('');
  
  tsCode.push('/**');
  tsCode.push(' * Get a CSS custom property name for a token');
  tsCode.push(' */');
  tsCode.push('export function getCSSVar(key: keyof DesignTokens): string {');
  tsCode.push('  return `var(--${key})`;');
  tsCode.push('}');
  tsCode.push('');
  
  // Generate metadata
  tsCode.push('export const metadata = {');
  tsCode.push(`  name: "${tokens.metadata.name}",`);
  tsCode.push(`  version: "${tokens.metadata.version}",`);
  tsCode.push(`  description: "${tokens.metadata.description}",`);
  tsCode.push(`  lastModified: "${tokens.metadata.lastModified}",`);
  tsCode.push(`  generated: "${new Date().toISOString()}",`);
  tsCode.push('};');
  
  return tsCode.join('\n');
}

// Main generation function
function generateTokens(): void {
  console.log('🎨 Generating Aetheris OS Design Tokens...');
  
  // Check if source file exists
  if (!fs.existsSync(CONFIG.sourceFile)) {
    console.error(`❌ Source file not found: ${CONFIG.sourceFile}`);
    process.exit(1);
  }
  
  // Load and parse YAML
  let tokens: DesignTokens;
  try {
    const yamlContent = fs.readFileSync(CONFIG.sourceFile, 'utf8');
    tokens = yaml.load(yamlContent) as DesignTokens;
  } catch (error) {
    console.error(`❌ Failed to parse YAML: ${error}`);
    process.exit(1);
  }
  
  // Ensure output directory exists
  ensureDir(CONFIG.outputDir);
  
  // Generate each format
  const generators = {
    css: generateCSS,
    json: generateJSON,
    rs: generateRust,
    ts: generateTypeScript,
  };
  
  for (const format of CONFIG.formats) {
    try {
      const content = generators[format](tokens);
      const filename = `tokens.${format}`;
      const filepath = path.join(CONFIG.outputDir, filename);
      
      fs.writeFileSync(filepath, content, 'utf8');
      console.log(`✅ Generated: ${filepath}`);
    } catch (error) {
      console.error(`❌ Failed to generate ${format}: ${error}`);
    }
  }
  
  // Generate additional files
  generateAdditionalFiles(tokens);
  
  console.log('🎉 Design tokens generation completed!');
}

// Generate additional utility files
function generateAdditionalFiles(tokens: DesignTokens): void {
  // Generate index files
  const indexTS = `export * from './tokens';\n`;
  fs.writeFileSync(path.join(CONFIG.outputDir, 'index.ts'), indexTS);
  console.log(`✅ Generated: ${path.join(CONFIG.outputDir, 'index.ts')}`);
  
  // Generate README
  const readme = `# Aetheris OS Design Tokens

This directory contains generated design tokens for Aetheris OS.

## Files

- \`tokens.css\` - CSS custom properties
- \`tokens.json\` - JSON format
- \`tokens.rs\` - Rust constants
- \`tokens.ts\` - TypeScript definitions and utilities
- \`index.ts\` - TypeScript exports

## Usage

### CSS
\`\`\`css
@import './tokens.css';

.my-component {
  color: var(--primary-500);
  padding: var(--spacing-4);
}
\`\`\`

### TypeScript
\`\`\`typescript
import { tokens, getToken, getCSSVar } from './tokens';

const primaryColor = getToken('primary-500');
const cssVar = getCSSVar('spacing-4');
\`\`\`

### Rust
\`\`\`rust
use aetheris_tokens::tokens::*;

let primary_color = PRIMARY_500;
\`\`\`

## Source

Generated from: \`${CONFIG.sourceFile}\`
Version: ${tokens.metadata.version}
Last Modified: ${tokens.metadata.lastModified}
Generated: ${new Date().toISOString()}
`;
  
  fs.writeFileSync(path.join(CONFIG.outputDir, 'README.md'), readme);
  console.log(`✅ Generated: ${path.join(CONFIG.outputDir, 'README.md')}`);
}

// CLI handling
function main(): void {
  const args = process.argv.slice(2);
  
  if (args.includes('--help') || args.includes('-h')) {
    console.log(`
Usage: generate-tokens [options]

Options:
  --help, -h     Show this help message
  --version      Show version information
  --watch        Watch for changes and regenerate

Environment Variables:
  TOKENS_SOURCE  Override source file (default: design/Design-Tokens.yaml)
  TOKENS_OUTPUT  Override output directory (default: ui/tokens)
`);
    process.exit(0);
  }
  
  if (args.includes('--version')) {
    console.log('Design Tokens Generator v1.0.0');
    process.exit(0);
  }
  
  if (args.includes('--watch')) {
    console.log('👀 Watching for changes...');
    fs.watchFile(CONFIG.sourceFile, () => {
      console.log('📝 Source file changed, regenerating...');
      generateTokens();
    });
  } else {
    generateTokens();
  }
}

// Run if called directly
if (require.main === module) {
  main();
}

export { generateTokens, generateCSS, generateJSON, generateRust, generateTypeScript };
