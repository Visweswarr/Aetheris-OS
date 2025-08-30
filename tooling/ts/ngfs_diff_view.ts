#!/usr/bin/env node
/**
 * NGFS Diff Viewer - Renders NGFS diffs in a colored tree format
 */

import * as fs from 'fs';
import * as path from 'path';

// Types for NGFS diff structures
interface DiffV1 {
  version: number;
  timestamp: string;
  old_snapshot: Uint8Array;
  new_snapshot: Uint8Array;
  added: Entry[];
  removed: Entry[];
  modified: ModEntry[];
  summary: DiffSummary;
}

interface Entry {
  path: string;
  cid_ngfs: Uint8Array;
  size: number;
  entry_type: EntryType;
  mode: number;
  mtime: number;
}

interface ModEntry {
  path: string;
  old_cid: Uint8Array;
  new_cid: Uint8Array;
  old_size: number;
  new_size: number;
  delta_size: number;
  mode: number;
  mtime: number;
}

interface DiffSummary {
  total_added: number;
  total_removed: number;
  total_modified: number;
  bytes_added: number;
  bytes_removed: number;
  bytes_delta: number;
}

enum EntryType {
  File = 1,
  Directory = 2,
  Symlink = 3,
  Special = 4,
}

// Color codes for terminal output
const Colors = {
  Reset: '\x1b[0m',
  Bright: '\x1b[1m',
  Red: '\x1b[31m',
  Green: '\x1b[32m',
  Yellow: '\x1b[33m',
  Blue: '\x1b[34m',
  Magenta: '\x1b[35m',
  Cyan: '\x1b[36m',
  White: '\x1b[37m',
  Gray: '\x1b[90m',
};

// Diff viewer class
class NgfsDiffViewer {
  private diff: DiffV1 | null = null;
  private showColors: boolean = true;
  private maxDepth: number = 10;
  private compactMode: boolean = false;
  
  constructor() {
    // Detect if colors are supported
    this.showColors = process.stdout.isTTY && !process.env.NO_COLOR;
  }
  
  /**
   * Load diff from file
   */
  public loadDiff(filePath: string): boolean {
    try {
      const fileContent = fs.readFileSync(filePath);
      
      // Try to parse as CBOR first, then JSON
      try {
        // For now, we'll use a placeholder since CBOR parsing isn't implemented
        // In a real implementation, you'd use a CBOR library
        this.diff = this.parseDiffContent(fileContent, filePath);
        return true;
      } catch (error) {
        console.error(`Error parsing diff file: ${error}`);
        return false;
      }
    } catch (error) {
      console.error(`Error reading file ${filePath}: ${error}`);
      return false;
    }
  }
  
  /**
   * Parse diff content (placeholder implementation)
   */
  private parseDiffContent(content: Buffer, filePath: string): DiffV1 {
    // This is a placeholder - in a real implementation, you'd parse CBOR
    // For now, create a mock diff structure for demonstration
    
    if (filePath.endsWith('.json')) {
      try {
        const jsonData = JSON.parse(content.toString());
        return this.validateAndTransform(jsonData);
      } catch (error) {
        throw new Error(`Invalid JSON: ${error}`);
      }
    } else {
      // Mock CBOR data for demonstration
      return this.createMockDiff();
    }
  }
  
  /**
   * Create a mock diff for demonstration
   */
  private createMockDiff(): DiffV1 {
    return {
      version: 1,
      timestamp: new Date().toISOString(),
      old_snapshot: new Uint8Array([1, 2, 3]),
      new_snapshot: new Uint8Array([4, 5, 6]),
      added: [
        {
          path: '/newfile.txt',
          cid_ngfs: new Uint8Array([10, 11, 12]),
          size: 1024,
          entry_type: EntryType.File,
          mode: 0o644,
          mtime: Date.now(),
        },
        {
          path: '/newdir',
          cid_ngfs: new Uint8Array([13, 14, 15]),
          size: 0,
          entry_type: EntryType.Directory,
          mode: 0o755,
          mtime: Date.now(),
        },
      ],
      removed: [
        {
          path: '/oldfile.txt',
          cid_ngfs: new Uint8Array([20, 21, 22]),
          size: 512,
          entry_type: EntryType.File,
          mode: 0o644,
          mtime: Date.now(),
        },
      ],
      modified: [
        {
          path: '/changed.txt',
          old_cid: new Uint8Array([30, 31, 32]),
          new_cid: new Uint8Array([33, 34, 35]),
          old_size: 256,
          new_size: 512,
          delta_size: 256,
          mode: 0o644,
          mtime: Date.now(),
        },
      ],
      summary: {
        total_added: 2,
        total_removed: 1,
        total_modified: 1,
        bytes_added: 1024,
        bytes_removed: 512,
        bytes_delta: 512,
      },
    };
  }
  
  /**
   * Validate and transform JSON data
   */
  private validateAndTransform(data: any): DiffV1 {
    // Basic validation
    if (!data.version || !data.timestamp || !data.summary) {
      throw new Error('Invalid diff structure: missing required fields');
    }
    
    // Transform the data to match our interface
    return {
      version: data.version,
      timestamp: data.timestamp,
      old_snapshot: this.hexToBytes(data.old_snapshot),
      new_snapshot: this.hexToBytes(data.new_snapshot),
      added: this.transformEntries(data.added || []),
      removed: this.transformEntries(data.removed || []),
      modified: this.transformModEntries(data.modified || []),
      summary: data.summary,
    };
  }
  
  /**
   * Transform entry data
   */
  private transformEntries(entries: any[]): Entry[] {
    return entries.map(entry => ({
      path: entry.path,
      cid_ngfs: this.hexToBytes(entry.cid_ngfs),
      size: entry.size || 0,
      entry_type: entry.entry_type || EntryType.File,
      mode: entry.mode || 0o644,
      mtime: entry.mtime || 0,
    }));
  }
  
  /**
   * Transform modified entry data
   */
  private transformModEntries(entries: any[]): ModEntry[] {
    return entries.map(entry => ({
      path: entry.path,
      old_cid: this.hexToBytes(entry.old_cid),
      new_cid: this.hexToBytes(entry.new_cid),
      old_size: entry.old_size || 0,
      new_size: entry.new_size || 0,
      delta_size: entry.delta_size || 0,
      mode: entry.mode || 0o644,
      mtime: entry.mtime || 0,
    }));
  }
  
  /**
   * Convert hex string to bytes
   */
  private hexToBytes(hex: string): Uint8Array {
    if (typeof hex !== 'string') {
      return new Uint8Array();
    }
    
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
      bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
    }
    return bytes;
  }
  
  /**
   * Render the diff in colored tree format
   */
  public render(): void {
    if (!this.diff) {
      console.error('No diff loaded');
      return;
    }
    
    console.log(this.colorize('NGFS Diff Viewer', Colors.Bright + Colors.Cyan));
    console.log(this.colorize('=' * 50, Colors.Gray));
    console.log();
    
    // Header information
    this.renderHeader();
    console.log();
    
    // Summary statistics
    this.renderSummary();
    console.log();
    
    // Tree view
    this.renderTree();
    console.log();
    
    // Detailed changes
    this.renderDetailedChanges();
  }
  
  /**
   * Render diff header
   */
  private renderHeader(): void {
    const diff = this.diff!;
    
    console.log(this.colorize('Diff Information:', Colors.Bright + Colors.White));
    console.log(`  Version:     ${diff.version}`);
    console.log(`  Timestamp:   ${diff.timestamp}`);
    console.log(`  Old Snapshot: ${this.bytesToHex(diff.old_snapshot)}`);
    console.log(`  New Snapshot: ${this.bytesToHex(diff.new_snapshot)}`);
  }
  
  /**
   * Render summary statistics
   */
  private renderSummary(): void {
    const summary = this.diff!.summary;
    
    console.log(this.colorize('Summary:', Colors.Bright + Colors.White));
    console.log(`  Added:       ${this.colorize(summary.total_added.toString(), Colors.Green)} entries`);
    console.log(`  Removed:     ${this.colorize(summary.total_removed.toString(), Colors.Red)} entries`);
    console.log(`  Modified:    ${this.colorize(summary.total_modified.toString(), Colors.Yellow)} entries`);
    console.log(`  Total:       ${summary.total_added + summary.total_removed + summary.total_modified} changes`);
    console.log();
    console.log('  Size Changes:');
    console.log(`    Added:     ${this.colorize(this.formatBytes(summary.bytes_added), Colors.Green)}`);
    console.log(`    Removed:   ${this.colorize(this.formatBytes(summary.bytes_removed), Colors.Red)}`);
    console.log(`    Net Delta: ${this.colorize(this.formatBytes(summary.bytes_delta), summary.bytes_delta >= 0 ? Colors.Green : Colors.Red)}`);
  }
  
  /**
   * Render tree view
   */
  private renderTree(): void {
    const diff = this.diff!;
    
    console.log(this.colorize('Tree View:', Colors.Bright + Colors.White));
    
    // Build path tree
    const tree = this.buildPathTree();
    
    // Render tree
    this.renderTreeNode(tree, '', 0);
  }
  
  /**
   * Build a tree structure from paths
   */
  private buildPathTree(): any {
    const tree: any = {};
    const diff = this.diff!;
    
    // Add added entries
    for (const entry of diff.added) {
      this.addPathToTree(tree, entry.path, 'added', entry);
    }
    
    // Add removed entries
    for (const entry of diff.removed) {
      this.addPathToTree(tree, entry.path, 'removed', entry);
    }
    
    // Add modified entries
    for (const entry of diff.modified) {
      this.addPathToTree(tree, entry.path, 'modified', entry);
    }
    
    return tree;
  }
  
  /**
   * Add a path to the tree
   */
  private addPathToTree(tree: any, pathStr: string, changeType: string, entry: any): void {
    const components = pathStr.split('/').filter(c => c !== '');
    let current = tree;
    
    for (let i = 0; i < components.length; i++) {
      const component = components[i];
      const isLast = i === components.length - 1;
      
      if (!current[component]) {
        current[component] = {
          type: isLast ? 'file' : 'directory',
          changes: [],
          children: {},
        };
      }
      
      if (isLast) {
        current[component].changes.push({
          type: changeType,
          entry: entry,
        });
      } else {
        if (!current[component].children) {
          current[component].children = {};
        }
        current = current[component].children;
      }
    }
  }
  
  /**
   * Render a tree node
   */
  private renderTreeNode(node: any, prefix: string, depth: number): void {
    if (depth > this.maxDepth) {
      console.log(`${prefix}${this.colorize('...', Colors.Gray)}`);
      return;
    }
    
    const entries = Object.entries(node);
    entries.sort(([a], [b]) => a.localeCompare(b));
    
    for (let i = 0; i < entries.length; i++) {
      const [name, data] = entries[i];
      const isLast = i === entries.length - 1;
      const connector = isLast ? '└── ' : '├── ';
      const nextPrefix = prefix + (isLast ? '    ' : '│   ');
      
      // Determine icon and color based on type and changes
      let icon = '📄';
      let color = Colors.White;
      
      if (data.type === 'directory') {
        icon = '📁';
        color = Colors.Blue;
      }
      
      // Apply change colors
      if (data.changes && data.changes.length > 0) {
        const changeTypes = data.changes.map((c: any) => c.type);
        if (changeTypes.includes('added')) {
          color = Colors.Green;
          icon = changeTypes.includes('removed') ? '🔄' : '➕';
        } else if (changeTypes.includes('removed')) {
          color = Colors.Red;
          icon = '➖';
        } else if (changeTypes.includes('modified')) {
          color = Colors.Yellow;
          icon = '✏️';
        }
      }
      
      // Render the node
      const nodeStr = `${prefix}${connector}${icon} ${name}`;
      console.log(this.colorize(nodeStr, color));
      
      // Render changes if any
      if (data.changes && data.changes.length > 0) {
        for (const change of data.changes) {
          const changeStr = `${nextPrefix}    ${this.getChangeSymbol(change.type)} ${this.getChangeDescription(change)}`;
          console.log(this.colorize(changeStr, this.getChangeColor(change.type)));
        }
      }
      
      // Render children
      if (data.children && Object.keys(data.children).length > 0) {
        this.renderTreeNode(data.children, nextPrefix, depth + 1);
      }
    }
  }
  
  /**
   * Get change symbol
   */
  private getChangeSymbol(changeType: string): string {
    switch (changeType) {
      case 'added': return '+';
      case 'removed': return '-';
      case 'modified': return '~';
      default: return '?';
    }
  }
  
  /**
   * Get change color
   */
  private getChangeColor(changeType: string): string {
    switch (changeType) {
      case 'added': return Colors.Green;
      case 'removed': return Colors.Red;
      case 'modified': return Colors.Yellow;
      default: return Colors.Gray;
    }
  }
  
  /**
   * Get change description
   */
  private getChangeDescription(change: any): string {
    const entry = change.entry;
    
    if (change.type === 'modified') {
      const deltaStr = entry.delta_size >= 0 ? `+${this.formatBytes(entry.delta_size)}` : this.formatBytes(entry.delta_size);
      return `${this.formatBytes(entry.old_size)} → ${this.formatBytes(entry.new_size)} (${deltaStr})`;
    } else {
      return this.formatBytes(entry.size);
    }
  }
  
  /**
   * Render detailed changes
   */
  private renderDetailedChanges(): void {
    const diff = this.diff!;
    
    console.log(this.colorize('Detailed Changes:', Colors.Bright + Colors.White));
    console.log();
    
    // Added entries
    if (diff.added.length > 0) {
      console.log(this.colorize('Added Files:', Colors.Green));
      for (const entry of diff.added) {
        const sizeStr = this.formatBytes(entry.size);
        const typeStr = this.getEntryTypeString(entry.entry_type);
        console.log(`  + ${entry.path} (${sizeStr}, ${typeStr})`);
      }
      console.log();
    }
    
    // Removed entries
    if (diff.removed.length > 0) {
      console.log(this.colorize('Removed Files:', Colors.Red));
      for (const entry of diff.removed) {
        const sizeStr = this.formatBytes(entry.size);
        const typeStr = this.getEntryTypeString(entry.entry_type);
        console.log(`  - ${entry.path} (${sizeStr}, ${typeStr})`);
      }
      console.log();
    }
    
    // Modified entries
    if (diff.modified.length > 0) {
      console.log(this.colorize('Modified Files:', Colors.Yellow));
      for (const entry of diff.modified) {
        const oldSizeStr = this.formatBytes(entry.old_size);
        const newSizeStr = this.formatBytes(entry.new_size);
        const deltaStr = entry.delta_size >= 0 ? `+${this.formatBytes(entry.delta_size)}` : this.formatBytes(entry.delta_size);
        console.log(`  ~ ${entry.path} (${oldSizeStr} → ${newSizeStr}, ${deltaStr})`);
      }
      console.log();
    }
  }
  
  /**
   * Get entry type string
   */
  private getEntryTypeString(type: EntryType): string {
    switch (type) {
      case EntryType.File: return 'file';
      case EntryType.Directory: return 'directory';
      case EntryType.Symlink: return 'symlink';
      case EntryType.Special: return 'special';
      default: return 'unknown';
    }
  }
  
  /**
   * Format bytes in human-readable format
   */
  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let unitIndex = 0;
    let value = bytes;
    
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }
    
    if (unitIndex === 0) {
      return `${value} ${units[unitIndex]}`;
    } else {
      return `${value.toFixed(1)} ${units[unitIndex]}`;
    }
  }
  
  /**
   * Convert bytes to hex string
   */
  private bytesToHex(bytes: Uint8Array): string {
    return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
  }
  
  /**
   * Colorize text if colors are enabled
   */
  private colorize(text: string, color: string): string {
    if (!this.showColors) {
      return text;
    }
    return `${color}${text}${Colors.Reset}`;
  }
  
  /**
   * Set options
   */
  public setOptions(options: { maxDepth?: number; compactMode?: boolean; showColors?: boolean }): void {
    if (options.maxDepth !== undefined) this.maxDepth = options.maxDepth;
    if (options.compactMode !== undefined) this.compactMode = options.compactMode;
    if (options.showColors !== undefined) this.showColors = options.showColors;
  }
}

// Main function
function main(): void {
  const args = process.argv.slice(2);
  
  if (args.length === 0 || args.includes('--help') || args.includes('-h')) {
    showHelp();
    process.exit(0);
  }
  
  const diffFile = args[0];
  
  if (!fs.existsSync(diffFile)) {
    console.error(`Error: Diff file not found: ${diffFile}`);
    process.exit(1);
  }
  
  const viewer = new NgfsDiffViewer();
  
  // Parse options
  const options: any = {};
  for (let i = 1; i < args.length; i++) {
    if (args[i] === '--no-colors') {
      options.showColors = false;
    } else if (args[i] === '--max-depth' && i + 1 < args.length) {
      options.maxDepth = parseInt(args[i + 1]);
      i++;
    } else if (args[i] === '--compact') {
      options.compactMode = true;
    }
  }
  
  viewer.setOptions(options);
  
  if (!viewer.loadDiff(diffFile)) {
    console.error('Failed to load diff file');
    process.exit(1);
  }
  
  viewer.render();
}

// Show help information
function showHelp(): void {
  console.log(`
NGFS Diff Viewer - Renders NGFS diffs in a colored tree format

Usage: ngfs_diff_view.ts <diff_file> [options]

Arguments:
  <diff_file>           Path to the diff file (.cbor or .json)

Options:
  --no-colors           Disable colored output
  --max-depth <n>       Maximum tree depth to display (default: 10)
  --compact             Compact display mode
  --help, -h            Show this help message

Examples:
  node ngfs_diff_view.js diff.cbor
  node ngfs_diff_view.js diff.json --no-colors
  node ngfs_diff_view.js diff.cbor --max-depth 5 --compact

Output:
  The viewer displays:
  - Diff header information
  - Summary statistics
  - Tree view with colored changes
  - Detailed change list
  
  Colors indicate change types:
  + Green: Added files/directories
  - Red: Removed files/directories
  ~ Yellow: Modified files
  📁 Blue: Directories
  📄 White: Files
`);
}

// Run main function if this file is executed directly
if (require.main === module) {
  main();
}

export { NgfsDiffViewer, DiffV1, Entry, ModEntry, DiffSummary, EntryType };
