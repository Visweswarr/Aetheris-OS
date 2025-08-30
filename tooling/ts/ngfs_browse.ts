#!/usr/bin/env node
/**
 * NGFS Tree Browser - Interactive TUI for exploring mounted NGFS trees
 */

import * as fs from 'fs';
import * as path from 'path';
import * as crypto from 'crypto';

// Types and interfaces
export interface BrowserStats {
  directories: number;
  files: number;
  totalSize: number;
  startTime: Date;
  
  addDirectory(): void;
  addFile(size: number): void;
  reset(): void;
}

export interface DirectoryEntry {
  name: string;
  type: 'directory';
  mode: number;
  mtime: Date;
}

export interface FileEntry {
  name: string;
  type: 'file';
  size: number;
  mode: number;
  mtime: Date;
}

export type Entry = DirectoryEntry | FileEntry;

export class NgfsBrowser {
  public mountPoint: string;
  public currentPath: string;
  public stats: BrowserStats;
  
  constructor(mountPoint: string) {
    if (!fs.existsSync(mountPoint)) {
      throw new Error(`Mount point does not exist: ${mountPoint}`);
    }
    
    this.mountPoint = mountPoint;
    this.currentPath = '/';
    this.stats = new BrowserStats();
  }
  
  /**
   * Navigate to a new path
   */
  public navigateTo(targetPath: string): void {
    let newPath: string;
    
    if (targetPath.startsWith('/')) {
      newPath = targetPath;
    } else if (targetPath === '..') {
      if (this.currentPath === '/') {
        throw new Error('Cannot navigate above root');
      }
      newPath = path.dirname(this.currentPath);
    } else {
      newPath = path.join(this.currentPath, targetPath);
    }
    
    // Normalize path
    newPath = path.normalize(newPath);
    
    // Ensure we don't escape the mount point
    const fullPath = path.join(this.mountPoint, newPath);
    if (!fullPath.startsWith(this.mountPoint)) {
      throw new Error('Cannot navigate above root');
    }
    
    if (!fs.existsSync(fullPath)) {
      throw new Error(`Path does not exist: ${targetPath}`);
    }
    
    const stat = fs.statSync(fullPath);
    if (!stat.isDirectory()) {
      throw new Error(`Path is not a directory: ${targetPath}`);
    }
    
    this.currentPath = newPath;
  }
  
  /**
   * List contents of current directory
   */
  public listDirectory(): Entry[] {
    try {
      const fullPath = path.join(this.mountPoint, this.currentPath);
      const entries = fs.readdirSync(fullPath);
      
      const result: Entry[] = [];
      
      for (const entry of entries) {
        const entryPath = path.join(fullPath, entry);
        const stat = fs.statSync(entryPath);
        
        if (stat.isDirectory()) {
          result.push({
            name: entry,
            type: 'directory',
            mode: stat.mode,
            mtime: stat.mtime
          });
          this.stats.addDirectory();
        } else {
          result.push({
            name: entry,
            type: 'file',
            size: stat.size,
            mode: stat.mode,
            mtime: stat.mtime
          });
          this.stats.addFile(stat.size);
        }
      }
      
      // Sort: directories first, then files, both alphabetically
      result.sort((a, b) => {
        if (a.type !== b.type) {
          return a.type === 'directory' ? -1 : 1;
        }
        return a.name.localeCompare(b.name);
      });
      
      return result;
    } catch (error) {
      throw new Error(`Failed to read directory: ${error}`);
    }
  }
  
  /**
   * Get file statistics
   */
  public getFileStats(filePath: string): fs.Stats {
    try {
      const fullPath = path.join(this.mountPoint, filePath);
      return fs.statSync(fullPath);
    } catch (error) {
      throw new Error(`Failed to get file stats: ${error}`);
    }
  }
  
  /**
   * Preview file contents
   */
  public previewFile(filePath: string): string {
    try {
      const fullPath = path.join(this.mountPoint, filePath);
      const stat = fs.statSync(fullPath);
      
      if (stat.isDirectory()) {
        return `Directory: ${filePath}\nMode: ${this.formatMode(stat.mode)}\nModified: ${this.formatDate(stat.mtime)}`;
      }
      
      const maxPreviewSize = 64 * 1024; // 64 KB
      
      if (stat.size > maxPreviewSize) {
        return `File: ${filePath}\nSize: ${this.formatSize(stat.size)}\nMode: ${this.formatMode(stat.mode)}\nModified: ${this.formatDate(stat.mtime)}\n\nFile too large for preview (${this.formatSize(stat.size)})`;
      }
      
      const content = fs.readFileSync(fullPath);
      const cid = this.generateCID(content);
      
      let preview = `File: ${filePath}\n`;
      preview += `Size: ${this.formatSize(stat.size)}\n`;
      preview += `Mode: ${this.formatMode(stat.mode)}\n`;
      preview += `Modified: ${this.formatDate(stat.mtime)}\n`;
      preview += `CID: ${cid}\n\n`;
      
      if (content.length > 0) {
        preview += 'Hex Dump (first 64 bytes):\n';
        const hexDump = this.generateHexDump(content.slice(0, 64));
        preview += hexDump + '\n\n';
        
        preview += 'ASCII:\n';
        const ascii = this.generateAsciiPreview(content.slice(0, 64));
        preview += ascii;
      }
      
      return preview;
    } catch (error) {
      return `Error reading file: ${error}`;
    }
  }
  
  /**
   * Generate content identifier
   */
  public generateCID(content: Buffer): string {
    const hash = crypto.createHash('sha256');
    hash.update(content);
    return hash.digest('hex');
  }
  
  /**
   * Generate hex dump
   */
  private generateHexDump(buffer: Buffer): string {
    let result = '';
    const bytesPerLine = 16;
    
    for (let i = 0; i < buffer.length; i += bytesPerLine) {
      const offset = i.toString(16).padStart(8, '0');
      result += `${offset}: `;
      
      const lineBytes = buffer.slice(i, i + bytesPerLine);
      const hex = Array.from(lineBytes).map(b => b.toString(16).padStart(2, '0')).join(' ');
      result += hex.padEnd(bytesPerLine * 3 - 1, ' ');
      
      result += '  ';
      
      const ascii = Array.from(lineBytes).map(b => {
        return (b >= 32 && b <= 126) ? String.fromCharCode(b) : '.';
      }).join('');
      result += ascii;
      
      result += '\n';
    }
    
    return result.trimEnd();
  }
  
  /**
   * Generate ASCII preview
   */
  private generateAsciiPreview(buffer: Buffer): string {
    return Array.from(buffer).map(b => {
      return (b >= 32 && b <= 126) ? String.fromCharCode(b) : '.';
    }).join('');
  }
  
  /**
   * Format file size
   */
  public formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    if (bytes === 1) return '1 B';
    
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    if (i === 0) return `${bytes} B`;
    
    const size = bytes / Math.pow(k, i);
    return `${size.toFixed(1)} ${sizes[i]}`;
  }
  
  /**
   * Format file mode
   */
  public formatMode(mode: number): string {
    const permissions = mode & 0o777;
    const type = mode & 0o170000;
    
    let result = '';
    
    // File type
    if (type === 0o40000) result += 'd';      // Directory
    else if (type === 0o100000) result += '-'; // Regular file
    else if (type === 0o120000) result += 'l'; // Symbolic link
    else if (type === 0o60000) result += 'b';  // Block device
    else if (type === 0o140000) result += 's'; // Socket
    else result += '-';
    
    // Permissions
    const owner = (permissions >> 6) & 0o7;
    const group = (permissions >> 3) & 0o7;
    const other = permissions & 0o7;
    
    result += this.formatPermissions(owner);
    result += this.formatPermissions(group);
    result += this.formatPermissions(other);
    
    return result;
  }
  
  /**
   * Format permissions for owner/group/other
   */
  private formatPermissions(perms: number): string {
    let result = '';
    result += (perms & 0o4) ? 'r' : '-';
    result += (perms & 0o2) ? 'w' : '-';
    result += (perms & 0o1) ? 'x' : '-';
    return result;
  }
  
  /**
   * Format date
   */
  public formatDate(date: Date): string {
    return date.toISOString().replace('T', ' ').substring(0, 19);
  }
  
  /**
   * Get elapsed time since start
   */
  public getElapsedTime(): number {
    return Date.now() - this.stats.startTime.getTime();
  }
  
  /**
   * Display current directory listing
   */
  public displayListing(): string {
    const entries = this.listDirectory();
    let output = `Current path: ${this.currentPath}\n`;
    output += `Entries: ${entries.length}\n\n`;
    
    if (entries.length === 0) {
      output += '(empty directory)\n';
      return output;
    }
    
    for (const entry of entries) {
      const icon = entry.type === 'directory' ? '📁' : '📄';
      const mode = this.formatMode(entry.mode);
      const date = this.formatDate(entry.mtime);
      
      if (entry.type === 'directory') {
        output += `${icon} ${entry.name}/                    [${mode.toString(8)}] ${date}\n`;
      } else {
        const size = this.formatSize(entry.size);
        output += `${icon} ${entry.name.padEnd(20)} [${mode.toString(8)}] ${size.padStart(8)}  ${date}\n`;
      }
    }
    
    return output;
  }
  
  /**
   * Display statistics
   */
  public displayStats(): string {
    const elapsed = this.getElapsedTime();
    let output = 'Browser Statistics\n';
    output += '==================\n';
    output += `Directories: ${this.stats.directories}\n`;
    output += `Files: ${this.stats.files}\n`;
    output += `Total Size: ${this.formatSize(this.stats.totalSize)}\n`;
    output += `Elapsed Time: ${elapsed}ms\n`;
    output += `Start Time: ${this.formatDate(this.stats.startTime)}\n`;
    return output;
  }
}

/**
 * Browser statistics tracker
 */
export class BrowserStats implements BrowserStats {
  public directories: number = 0;
  public files: number = 0;
  public totalSize: number = 0;
  public startTime: Date = new Date();
  
  public addDirectory(): void {
    this.directories++;
  }
  
  public addFile(size: number): void {
    this.files++;
    this.totalSize += size;
  }
  
  public reset(): void {
    this.directories = 0;
    this.files = 0;
    this.totalSize = 0;
    this.startTime = new Date();
  }
}

/**
 * Main function for CLI usage
 */
function main(): void {
  const args = process.argv.slice(2);
  
  if (args.length === 0 || args.includes('--help') || args.includes('-h')) {
    console.log(`
NGFS Tree Browser - Interactive TUI for exploring mounted NGFS trees

Usage: ngfs-browse [options] --mnt <mount-point>

Options:
  --mnt <path>           Mount point path (required)
  --verbose, -v          Verbose output
  --json                 JSON output mode
  --help, -h            Show this help message

Examples:
  ngfs-browse --mnt ./mnt
  ngfs-browse --mnt /mnt/ngfs --verbose
  ngfs-browse --mnt ./mnt --json

Navigation:
  Arrow Keys             Navigate through entries
  Enter                  Open directories or preview files
  Backspace             Go to parent directory
  q                     Quit the browser
  h                     Show help
`);
    process.exit(0);
  }
  
  // Parse arguments
  let mountPoint = '';
  let verbose = false;
  let jsonMode = false;
  
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--mnt' && i + 1 < args.length) {
      mountPoint = args[i + 1];
      i++;
    } else if (args[i] === '--verbose' || args[i] === '-v') {
      verbose = true;
    } else if (args[i] === '--json') {
      jsonMode = true;
    }
  }
  
  if (!mountPoint) {
    console.error('Error: --mnt option is required');
    process.exit(1);
  }
  
  try {
    const browser = new NgfsBrowser(mountPoint);
    
    if (jsonMode) {
      // JSON output mode
      const entries = browser.listDirectory();
      const stats = browser.stats;
      
      const output = {
        mount_point: mountPoint,
        current_path: browser.currentPath,
        entries: entries.map(entry => ({
          name: entry.name,
          type: entry.type,
          mode: entry.mode,
          mtime: entry.mtime.toISOString(),
          ...(entry.type === 'file' && { size: entry.size })
        })),
        statistics: {
          directories: stats.directories,
          files: stats.files,
          total_size: stats.totalSize,
          start_time: stats.startTime.toISOString()
        }
      };
      
      console.log(JSON.stringify(output, null, 2));
    } else {
      // Interactive mode
      console.log('NGFS Tree Browser');
      console.log('==================\n');
      
      if (verbose) {
        console.log(browser.displayStats());
        console.log();
      }
      
      console.log(browser.displayListing());
      
      if (verbose) {
        console.log('\nUse arrow keys to navigate, Enter to open, q to quit');
      }
    }
  } catch (error) {
    console.error(`Error: ${error}`);
    process.exit(1);
  }
}

// Run main function if this file is executed directly
if (require.main === module) {
  main();
}
