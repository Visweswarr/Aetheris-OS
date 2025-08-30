#!/usr/bin/env node
/**
 * Tests for ngfs-browse.ts - NGFS TUI browser for mounted trees
 */

import { NgfsBrowser, DirectoryEntry, FileEntry, BrowserStats } from '../ngfs_browse';

// Mock filesystem operations
const mockFs = {
  readdirSync: jest.fn(),
  statSync: jest.fn(),
  readFileSync: jest.fn(),
  existsSync: jest.fn(),
};

jest.mock('fs', () => mockFs);

describe('NgfsBrowser', () => {
  let browser: NgfsBrowser;
  
  beforeEach(() => {
    browser = new NgfsBrowser('/test/mount');
    jest.clearAllMocks();
  });
  
  describe('Initialization', () => {
    it('should initialize with mount point', () => {
      expect(browser.mountPoint).toBe('/test/mount');
      expect(browser.currentPath).toBe('/');
      expect(browser.stats).toEqual({
        directories: 0,
        files: 0,
        totalSize: 0,
        startTime: expect.any(Date)
      });
    });
    
    it('should validate mount point exists', () => {
      mockFs.existsSync.mockReturnValue(false);
      
      expect(() => new NgfsBrowser('/nonexistent')).toThrow('Mount point does not exist');
    });
  });
  
  describe('Path Navigation', () => {
    it('should navigate to valid paths', () => {
      mockFs.existsSync.mockReturnValue(true);
      mockFs.statSync.mockReturnValue({ isDirectory: () => true });
      
      browser.navigateTo('/subdir');
      expect(browser.currentPath).toBe('/subdir');
    });
    
    it('should reject navigation to files', () => {
      mockFs.existsSync.mockReturnValue(true);
      mockFs.statSync.mockReturnValue({ isDirectory: () => false });
      
      expect(() => browser.navigateTo('/file.txt')).toThrow('Path is not a directory');
    });
    
    it('should reject navigation to nonexistent paths', () => {
      mockFs.existsSync.mockReturnValue(false);
      
      expect(() => browser.navigateTo('/nonexistent')).toThrow('Path does not exist');
    });
    
    it('should handle relative paths', () => {
      mockFs.existsSync.mockReturnValue(true);
      mockFs.statSync.mockReturnValue({ isDirectory: () => true });
      
      browser.navigateTo('subdir');
      expect(browser.currentPath).toBe('/subdir');
    });
    
    it('should handle parent directory navigation', () => {
      browser.currentPath = '/subdir/nested';
      mockFs.existsSync.mockReturnValue(true);
      mockFs.statSync.mockReturnValue({ isDirectory: () => true });
      
      browser.navigateTo('..');
      expect(browser.currentPath).toBe('/subdir');
    });
    
    it('should prevent escaping root', () => {
      expect(() => browser.navigateTo('../..')).toThrow('Cannot navigate above root');
    });
  });
  
  describe('Directory Listing', () => {
    it('should list directory contents', () => {
      const mockEntries = [
        { name: 'file1.txt', isDirectory: false },
        { name: 'dir1', isDirectory: true },
        { name: 'file2.txt', isDirectory: false }
      ];
      
      mockFs.readdirSync.mockReturnValue(mockEntries.map(e => e.name));
      mockFs.statSync.mockImplementation((path) => {
        const name = path.split('/').pop();
        const entry = mockEntries.find(e => e.name === name);
        return {
          isDirectory: () => entry?.isDirectory || false,
          size: entry?.isDirectory ? 0 : 1024,
          mode: 0o644,
          mtime: new Date('2024-01-01T12:00:00Z')
        };
      });
      
      const listing = browser.listDirectory();
      
      expect(listing).toHaveLength(3);
      expect(listing[0].name).toBe('dir1'); // Directories first
      expect(listing[1].name).toBe('file1.txt');
      expect(listing[2].name).toBe('file2.txt');
    });
    
    it('should handle empty directories', () => {
      mockFs.readdirSync.mockReturnValue([]);
      
      const listing = browser.listDirectory();
      expect(listing).toHaveLength(0);
    });
    
    it('should sort entries correctly', () => {
      const mockEntries = [
        { name: 'z_file.txt', isDirectory: false },
        { name: 'a_dir', isDirectory: true },
        { name: 'm_file.txt', isDirectory: false },
        { name: 'b_dir', isDirectory: true }
      ];
      
      mockFs.readdirSync.mockReturnValue(mockEntries.map(e => e.name));
      mockFs.statSync.mockImplementation((path) => {
        const name = path.split('/').pop();
        const entry = mockEntries.find(e => e.name === name);
        return {
          isDirectory: () => entry?.isDirectory || false,
          size: entry?.isDirectory ? 0 : 1024,
          mode: 0o644,
          mtime: new Date('2024-01-01T12:00:00Z')
        };
      });
      
      const listing = browser.listDirectory();
      
      expect(listing[0].name).toBe('a_dir');
      expect(listing[1].name).toBe('b_dir');
      expect(listing[2].name).toBe('m_file.txt');
      expect(listing[3].name).toBe('z_file.txt');
    });
    
    it('should handle read errors gracefully', () => {
      mockFs.readdirSync.mockImplementation(() => {
        throw new Error('Permission denied');
      });
      
      expect(() => browser.listDirectory()).toThrow('Failed to read directory');
    });
  });
  
  describe('File Information', () => {
    it('should get file stats', () => {
      const mockStats = {
        size: 2048,
        mode: 0o644,
        mtime: new Date('2024-01-01T12:00:00Z'),
        isDirectory: () => false
      };
      
      mockFs.statSync.mockReturnValue(mockStats);
      
      const stats = browser.getFileStats('/test/file.txt');
      
      expect(stats.size).toBe(2048);
      expect(stats.mode).toBe(0o644);
      expect(stats.mtime).toEqual(mockStats.mtime);
    });
    
    it('should handle stat errors', () => {
      mockFs.statSync.mockImplementation(() => {
        throw new Error('File not found');
      });
      
      expect(() => browser.getFileStats('/nonexistent')).toThrow('Failed to get file stats');
    });
  });
  
  describe('File Preview', () => {
    it('should generate hex dump for small files', () => {
      const fileContent = Buffer.from('Hello, World!', 'utf8');
      mockFs.readFileSync.mockReturnValue(fileContent);
      mockFs.statSync.mockReturnValue({
        size: fileContent.length,
        isDirectory: () => false
      });
      
      const preview = browser.previewFile('/test/file.txt');
      
      expect(preview).toContain('48 65 6c 6c 6f'); // "Hello" in hex
      expect(preview).toContain('Hello, World!'); // ASCII representation
    });
    
    it('should truncate large files', () => {
      const largeContent = Buffer.alloc(1024 * 1024); // 1MB
      mockFs.readFileSync.mockReturnValue(largeContent);
      mockFs.statSync.mockReturnValue({
        size: largeContent.length,
        isDirectory: () => false
      });
      
      const preview = browser.previewFile('/test/large.txt');
      
      expect(preview).toContain('File too large for preview');
      expect(preview).toContain('Size: 1,048,576 bytes');
    });
    
    it('should handle read errors', () => {
      mockFs.readFileSync.mockImplementation(() => {
        throw new Error('Permission denied');
      });
      mockFs.statSync.mockReturnValue({
        size: 100,
        isDirectory: () => false
      });
      
      const preview = browser.previewFile('/test/file.txt');
      
      expect(preview).toContain('Error reading file');
    });
    
    it('should skip preview for directories', () => {
      mockFs.statSync.mockReturnValue({
        size: 0,
        isDirectory: () => true
      });
      
      const preview = browser.previewFile('/test/dir');
      
      expect(preview).toContain('Directory');
    });
  });
  
  describe('CID Generation', () => {
    it('should generate deterministic CIDs', () => {
      const content1 = Buffer.from('test content', 'utf8');
      const content2 = Buffer.from('test content', 'utf8');
      
      const cid1 = browser.generateCID(content1);
      const cid2 = browser.generateCID(content2);
      
      expect(cid1).toBe(cid2);
    });
    
    it('should generate different CIDs for different content', () => {
      const content1 = Buffer.from('content 1', 'utf8');
      const content2 = Buffer.from('content 2', 'utf8');
      
      const cid1 = browser.generateCID(content1);
      const cid2 = browser.generateCID(content2);
      
      expect(cid1).not.toBe(cid2);
    });
    
    it('should handle empty content', () => {
      const emptyContent = Buffer.alloc(0);
      const cid = browser.generateCID(emptyContent);
      
      expect(cid).toBeDefined();
      expect(cid.length).toBeGreaterThan(0);
    });
  });
  
  describe('Statistics Tracking', () => {
    it('should track directory count', () => {
      const mockEntries = [
        { name: 'dir1', isDirectory: true },
        { name: 'dir2', isDirectory: true },
        { name: 'file1.txt', isDirectory: false }
      ];
      
      mockFs.readdirSync.mockReturnValue(mockEntries.map(e => e.name));
      mockFs.statSync.mockImplementation((path) => {
        const name = path.split('/').pop();
        const entry = mockEntries.find(e => e.name === name);
        return {
          isDirectory: () => entry?.isDirectory || false,
          size: entry?.isDirectory ? 0 : 1024,
          mode: 0o644,
          mtime: new Date('2024-01-01T12:00:00Z')
        };
      });
      
      browser.listDirectory();
      
      expect(browser.stats.directories).toBe(2);
      expect(browser.stats.files).toBe(1);
    });
    
    it('should track total file size', () => {
      const mockEntries = [
        { name: 'file1.txt', isDirectory: false, size: 1024 },
        { name: 'file2.txt', isDirectory: false, size: 2048 },
        { name: 'dir1', isDirectory: true, size: 0 }
      ];
      
      mockFs.readdirSync.mockReturnValue(mockEntries.map(e => e.name));
      mockFs.statSync.mockImplementation((path) => {
        const name = path.split('/').pop();
        const entry = mockEntries.find(e => e.name === name);
        return {
          isDirectory: () => entry?.isDirectory || false,
          size: entry?.size || 0,
          mode: 0o644,
          mtime: new Date('2024-01-01T12:00:00Z')
        };
      });
      
      browser.listDirectory();
      
      expect(browser.stats.totalSize).toBe(3072);
    });
    
    it('should calculate elapsed time', () => {
      const startTime = new Date('2024-01-01T12:00:00Z');
      browser.stats.startTime = startTime;
      
      const elapsed = browser.getElapsedTime();
      expect(elapsed).toBeGreaterThan(0);
    });
  });
  
  describe('Display Formatting', () => {
    it('should format file sizes correctly', () => {
      expect(browser.formatSize(1024)).toBe('1.0 KB');
      expect(browser.formatSize(1024 * 1024)).toBe('1.0 MB');
      expect(browser.formatSize(1024 * 1024 * 1024)).toBe('1.0 GB');
      expect(browser.formatSize(500)).toBe('500 B');
    });
    
    it('should format file modes correctly', () => {
      expect(browser.formatMode(0o644)).toBe('-rw-r--r--');
      expect(browser.formatMode(0o755)).toBe('-rwxr-xr-x');
      expect(browser.formatMode(0o400)).toBe('-r--------');
      expect(browser.formatMode(0o600)).toBe('-rw-------');
    });
    
    it('should format dates correctly', () => {
      const date = new Date('2024-01-01T12:00:00Z');
      const formatted = browser.formatDate(date);
      
      expect(formatted).toMatch(/\d{4}-\d{2}-\d{2}/);
      expect(formatted).toMatch(/\d{2}:\d{2}:\d{2}/);
    });
  });
  
  describe('Error Handling', () => {
    it('should handle filesystem errors gracefully', () => {
      mockFs.readdirSync.mockImplementation(() => {
        throw new Error('ENOENT: no such file or directory');
      });
      
      expect(() => browser.listDirectory()).toThrow('Failed to read directory');
    });
    
    it('should handle permission errors', () => {
      mockFs.readdirSync.mockImplementation(() => {
        throw new Error('EACCES: permission denied');
      });
      
      expect(() => browser.listDirectory()).toThrow('Failed to read directory');
    });
    
    it('should handle network filesystem errors', () => {
      mockFs.readdirSync.mockImplementation(() => {
        throw new Error('ENOTCONN: transport endpoint is not connected');
      });
      
      expect(() => browser.listDirectory()).toThrow('Failed to read directory');
    });
  });
  
  describe('Integration Tests', () => {
    it('should handle complex directory structures', () => {
      const mockStructure = {
        '/': ['dir1', 'file1.txt'],
        '/dir1': ['subdir', 'file2.txt'],
        '/dir1/subdir': ['file3.txt']
      };
      
      mockFs.existsSync.mockReturnValue(true);
      mockFs.statSync.mockImplementation((path) => {
        const isDir = mockStructure[path] !== undefined;
        return {
          isDirectory: () => isDir,
          size: isDir ? 0 : 1024,
          mode: 0o644,
          mtime: new Date('2024-01-01T12:00:00Z')
        };
      });
      
      mockFs.readdirSync.mockImplementation((path) => {
        return mockStructure[path] || [];
      });
      
      // Navigate through the structure
      browser.navigateTo('/dir1');
      let listing = browser.listDirectory();
      expect(listing).toHaveLength(2);
      
      browser.navigateTo('subdir');
      listing = browser.listDirectory();
      expect(listing).toHaveLength(1);
    });
    
    it('should maintain state across operations', () => {
      mockFs.existsSync.mockReturnValue(true);
      mockFs.statSync.mockReturnValue({ isDirectory: () => true });
      
      browser.navigateTo('/subdir');
      expect(browser.currentPath).toBe('/subdir');
      
      browser.navigateTo('nested');
      expect(browser.currentPath).toBe('/subdir/nested');
      
      browser.navigateTo('..');
      expect(browser.currentPath).toBe('/subdir');
    });
  });
});

describe('Utility Functions', () => {
  describe('formatSize', () => {
    it('should handle edge cases', () => {
      const browser = new NgfsBrowser('/test');
      
      expect(browser.formatSize(0)).toBe('0 B');
      expect(browser.formatSize(1)).toBe('1 B');
      expect(browser.formatSize(1023)).toBe('1023 B');
      expect(browser.formatSize(1024)).toBe('1.0 KB');
    });
  });
  
  describe('formatMode', () => {
    it('should handle special modes', () => {
      const browser = new NgfsBrowser('/test');
      
      expect(browser.formatMode(0o777)).toBe('-rwxrwxrwx');
      expect(browser.formatMode(0o000)).toBe('---------');
      expect(browser.formatMode(0o111)).toBe('--x--x--x');
      expect(browser.formatMode(0o222)).toBe('-w--w--w-');
      expect(browser.formatMode(0o444)).toBe('-r--r--r--');
    });
  });
  
  describe('generateCID', () => {
    it('should produce consistent results', () => {
      const browser = new NgfsBrowser('/test');
      const content = Buffer.from('test content', 'utf8');
      
      const cid1 = browser.generateCID(content);
      const cid2 = browser.generateCID(content);
      
      expect(cid1).toBe(cid2);
      expect(typeof cid1).toBe('string');
      expect(cid1.length).toBeGreaterThan(0);
    });
  });
});

describe('BrowserStats', () => {
  it('should track statistics correctly', () => {
    const stats = new BrowserStats();
    
    expect(stats.directories).toBe(0);
    expect(stats.files).toBe(0);
    expect(stats.totalSize).toBe(0);
    expect(stats.startTime).toBeInstanceOf(Date);
  });
  
  it('should update statistics', () => {
    const stats = new BrowserStats();
    
    stats.addDirectory();
    stats.addFile(1024);
    stats.addFile(2048);
    
    expect(stats.directories).toBe(1);
    expect(stats.files).toBe(2);
    expect(stats.totalSize).toBe(3072);
  });
  
  it('should reset statistics', () => {
    const stats = new BrowserStats();
    
    stats.addDirectory();
    stats.addFile(1024);
    
    stats.reset();
    
    expect(stats.directories).toBe(0);
    expect(stats.files).toBe(0);
    expect(stats.totalSize).toBe(0);
  });
});

describe('DirectoryEntry', () => {
  it('should create directory entries correctly', () => {
    const entry = new DirectoryEntry('testdir', 0o755, new Date());
    
    expect(entry.name).toBe('testdir');
    expect(entry.mode).toBe(0o755);
    expect(entry.mtime).toBeInstanceOf(Date);
    expect(entry.type).toBe('directory');
  });
});

describe('FileEntry', () => {
  it('should create file entries correctly', () => {
    const entry = new FileEntry('test.txt', 1024, 0o644, new Date());
    
    expect(entry.name).toBe('test.txt');
    expect(entry.size).toBe(1024);
    expect(entry.mode).toBe(0o644);
    expect(entry.mtime).toBeInstanceOf(Date);
    expect(entry.type).toBe('file');
  });
});
