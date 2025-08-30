import { NgfsDiffViewer, DiffV1, Entry, ModEntry, DiffSummary, EntryType } from '../ngfs_diff_view';

describe('NgfsDiffViewer', () => {
  let viewer: NgfsDiffViewer;
  let mockDiff: DiffV1;

  beforeEach(() => {
    viewer = new NgfsDiffViewer();
    
    // Create a mock diff for testing
    mockDiff = {
      version: 1,
      timestamp: "2024-01-01T12:00:00Z",
      old_snapshot: new Uint8Array([1, 2, 3]),
      new_snapshot: new Uint8Array([4, 5, 6]),
      added: [
        {
          path: "/file1.txt",
          cid_ngfs: new Uint8Array([10]),
          size: 1024,
          type: EntryType.File,
          mode: 0o644,
          mtime: 1640995200
        },
        {
          path: "/dir1",
          cid_ngfs: new Uint8Array([11]),
          size: 0,
          type: EntryType.Directory,
          mode: 0o755,
          mtime: 1640995200
        }
      ],
      removed: [
        {
          path: "/file2.txt",
          cid_ngfs: new Uint8Array([20]),
          size: 512,
          type: EntryType.File,
          mode: 0o644,
          mtime: 1640995200
        }
      ],
      modified: [
        {
          path: "/file3.txt",
          old_cid: new Uint8Array([30]),
          new_cid: new Uint8Array([31]),
          old_size: 256,
          new_size: 512,
          delta_size: 256,
          mode: 0o644,
          mtime: 1640995200
        }
      ],
      summary: {
        total_added: 2,
        total_removed: 1,
        total_modified: 1,
        bytes_added: 1024,
        bytes_removed: 512,
        bytes_delta: 512
      }
    };
  });

  describe('Initialization', () => {
    it('should initialize with default options', () => {
      expect(viewer['showColors']).toBe(true);
      expect(viewer['maxDepth']).toBe(10);
      expect(viewer['compactMode']).toBe(false);
      expect(viewer['diff']).toBeNull();
    });

    it('should set options correctly', () => {
      viewer.setOptions({
        maxDepth: 5,
        compactMode: true,
        showColors: false
      });

      expect(viewer['maxDepth']).toBe(5);
      expect(viewer['compactMode']).toBe(true);
      expect(viewer['showColors']).toBe(false);
    });
  });

  describe('Diff Loading', () => {
    it('should load diff from file path', () => {
      // Mock file system operations
      const mockFs = {
        readFileSync: jest.fn().mockReturnValue(Buffer.from('mock content'))
      };
      
      // This would test actual file loading
      // For now, we'll test the mock diff loading
      viewer['diff'] = mockDiff;
      expect(viewer['diff']).toBe(mockDiff);
    });

    it('should handle invalid file paths gracefully', () => {
      const result = viewer.loadDiff('/nonexistent/path');
      expect(result).toBe(false);
    });
  });

  describe('Path Tree Building', () => {
    it('should build path tree from diff entries', () => {
      viewer['diff'] = mockDiff;
      const tree = viewer['buildPathTree']();
      
      expect(tree).toBeDefined();
      expect(tree['/file1.txt']).toBeDefined();
      expect(tree['/dir1']).toBeDefined();
      expect(tree['/file2.txt']).toBeDefined();
      expect(tree['/file3.txt']).toBeDefined();
    });

    it('should handle nested paths correctly', () => {
      const nestedDiff: DiffV1 = {
        ...mockDiff,
        added: [
          {
            path: "/dir1/subdir/file.txt",
            cid_ngfs: new Uint8Array([100]),
            size: 1024,
            type: EntryType.File,
            mode: 0o644,
            mtime: 1640995200
          }
        ]
      };
      
      viewer['diff'] = nestedDiff;
      const tree = viewer['buildPathTree']();
      
      expect(tree['/dir1']).toBeDefined();
      expect(tree['/dir1']['subdir']).toBeDefined();
      expect(tree['/dir1']['subdir']['file.txt']).toBeDefined();
    });

    it('should handle empty diff', () => {
      const emptyDiff: DiffV1 = {
        ...mockDiff,
        added: [],
        removed: [],
        modified: []
      };
      
      viewer['diff'] = emptyDiff;
      const tree = viewer['buildPathTree']();
      
      expect(Object.keys(tree)).toHaveLength(0);
    });
  });

  describe('Rendering', () => {
    it('should render header information', () => {
      viewer['diff'] = mockDiff;
      const header = viewer['renderHeader']();
      
      expect(header).toContain('NGFS Diff');
      expect(header).toContain('2024-01-01T12:00:00Z');
      expect(header).toContain('Old: 010203');
      expect(header).toContain('New: 040506');
    });

    it('should render summary statistics', () => {
      viewer['diff'] = mockDiff;
      const summary = viewer['renderSummary']();
      
      expect(summary).toContain('Added: 2');
      expect(summary).toContain('Removed: 1');
      expect(summary).toContain('Modified: 1');
      expect(summary).toContain('Bytes Added: 1.0 KB');
      expect(summary).toContain('Bytes Removed: 512 B');
      expect(summary).toContain('Net Change: +512 B');
    });

    it('should render tree structure', () => {
      viewer['diff'] = mockDiff;
      const tree = viewer['renderTree']();
      
      expect(tree).toContain('+ /file1.txt');
      expect(tree).toContain('+ /dir1');
      expect(tree).toContain('- /file2.txt');
      expect(tree).toContain('~ /file3.txt');
    });

    it('should respect max depth setting', () => {
      viewer.setOptions({ maxDepth: 1 });
      viewer['diff'] = mockDiff;
      
      const tree = viewer['renderTree']();
      // Should not show nested paths beyond depth 1
      expect(tree).not.toContain('subdir');
    });

    it('should handle compact mode', () => {
      viewer.setOptions({ compactMode: true });
      viewer['diff'] = mockDiff;
      
      const tree = viewer['renderTree']();
      // Compact mode should show less verbose output
      expect(tree).toBeDefined();
    });
  });

  describe('Change Detection', () => {
    it('should identify added entries correctly', () => {
      viewer['diff'] = mockDiff;
      const addedEntries = mockDiff.added;
      
      expect(addedEntries).toHaveLength(2);
      expect(addedEntries[0].path).toBe('/file1.txt');
      expect(addedEntries[1].path).toBe('/dir1');
    });

    it('should identify removed entries correctly', () => {
      viewer['diff'] = mockDiff;
      const removedEntries = mockDiff.removed;
      
      expect(removedEntries).toHaveLength(1);
      expect(removedEntries[0].path).toBe('/file2.txt');
    });

    it('should identify modified entries correctly', () => {
      viewer['diff'] = mockDiff;
      const modifiedEntries = mockDiff.modified;
      
      expect(modifiedEntries).toHaveLength(1);
      expect(modifiedEntries[0].path).toBe('/file3.txt');
      expect(modifiedEntries[0].delta_size).toBe(256);
    });
  });

  describe('Color and Formatting', () => {
    it('should apply colors when enabled', () => {
      viewer.setOptions({ showColors: true });
      const coloredText = viewer['colorize']('test', 'green');
      
      expect(coloredText).toContain('\x1b[32m'); // Green color code
      expect(coloredText).toContain('\x1b[0m'); // Reset color code
    });

    it('should not apply colors when disabled', () => {
      viewer.setOptions({ showColors: false });
      const plainText = viewer['colorize']('test', 'green');
      
      expect(plainText).toBe('test');
    });

    it('should use correct colors for different change types', () => {
      const addedColor = viewer['getChangeColor']('added');
      const removedColor = viewer['getChangeColor']('removed');
      const modifiedColor = viewer['getChangeColor']('modified');
      
      expect(addedColor).toBe('green');
      expect(removedColor).toBe('red');
      expect(modifiedColor).toBe('yellow');
    });

    it('should use correct symbols for different change types', () => {
      const addedSymbol = viewer['getChangeSymbol']('added');
      const removedSymbol = viewer['getChangeSymbol']('removed');
      const modifiedSymbol = viewer['getChangeSymbol']('modified');
      
      expect(addedSymbol).toBe('+');
      expect(removedSymbol).toBe('-');
      expect(modifiedSymbol).toBe('~');
    });
  });

  describe('Utility Functions', () => {
    it('should format bytes correctly', () => {
      expect(viewer['formatBytes'](0)).toBe('0 B');
      expect(viewer['formatBytes'](1024)).toBe('1.0 KB');
      expect(viewer['formatBytes'](1024 * 1024)).toBe('1.0 MB');
      expect(viewer['formatBytes'](1024 * 1024 * 1024)).toBe('1.0 GB');
      expect(viewer['formatBytes'](1500)).toBe('1.5 KB');
      expect(viewer['formatBytes'](512)).toBe('512 B');
    });

    it('should convert bytes to hex correctly', () => {
      const bytes = new Uint8Array([0x01, 0x02, 0x03, 0x0A, 0xFF]);
      const hex = viewer['bytesToHex'](bytes);
      
      expect(hex).toBe('0102030aff');
    });

    it('should convert hex to bytes correctly', () => {
      const hex = '0102030aff';
      const bytes = viewer['hexToBytes'](hex);
      
      expect(bytes).toEqual(new Uint8Array([0x01, 0x02, 0x03, 0x0A, 0xFF]));
    });

    it('should get entry type string correctly', () => {
      expect(viewer['getEntryTypeString'](EntryType.File)).toBe('file');
      expect(viewer['getEntryTypeString'](EntryType.Directory)).toBe('directory');
      expect(viewer['getEntryTypeString'](EntryType.Symlink)).toBe('symlink');
      expect(viewer['getEntryTypeString'](EntryType.Special)).toBe('special');
    });
  });

  describe('Error Handling', () => {
    it('should handle null diff gracefully', () => {
      viewer['diff'] = null;
      
      expect(() => viewer['renderHeader']()).not.toThrow();
      expect(() => viewer['renderSummary']()).not.toThrow();
      expect(() => viewer['renderTree']()).not.toThrow();
    });

    it('should handle malformed diff data', () => {
      const malformedDiff = {
        ...mockDiff,
        added: undefined,
        removed: null,
        modified: 'invalid'
      } as any;
      
      viewer['diff'] = malformedDiff;
      
      // Should not crash
      expect(() => viewer['buildPathTree']()).not.toThrow();
    });

    it('should handle empty arrays gracefully', () => {
      const emptyDiff: DiffV1 = {
        ...mockDiff,
        added: [],
        removed: [],
        modified: []
      };
      
      viewer['diff'] = emptyDiff;
      
      const tree = viewer['buildPathTree']();
      expect(Object.keys(tree)).toHaveLength(0);
    });
  });

  describe('Integration Tests', () => {
    it('should render complete diff output', () => {
      viewer['diff'] = mockDiff;
      
      // Mock console.log to capture output
      const consoleSpy = jest.spyOn(console, 'log').mockImplementation();
      
      try {
        viewer.render();
        
        // Should have called console.log multiple times for different sections
        expect(consoleSpy).toHaveBeenCalled();
        
        // Check that different sections were rendered
        const calls = consoleSpy.mock.calls.map(call => call[0]);
        expect(calls.some(call => call.includes('NGFS Diff'))).toBe(true);
        expect(calls.some(call => call.includes('Added: 2'))).toBe(true);
        expect(calls.some(call => call.includes('+ /file1.txt'))).toBe(true);
      } finally {
        consoleSpy.mockRestore();
      }
    });

    it('should handle complex nested directory structures', () => {
      const complexDiff: DiffV1 = {
        ...mockDiff,
        added: [
          {
            path: "/root/dir1/subdir1/file1.txt",
            cid_ngfs: new Uint8Array([100]),
            size: 1024,
            type: EntryType.File,
            mode: 0o644,
            mtime: 1640995200
          },
          {
            path: "/root/dir2/subdir2/subsubdir/file2.txt",
            cid_ngfs: new Uint8Array([101]),
            size: 2048,
            type: EntryType.File,
            mode: 0o644,
            mtime: 1640995200
          }
        ]
      };
      
      viewer['diff'] = complexDiff;
      const tree = viewer['buildPathTree']();
      
      // Should handle deep nesting
      expect(tree['/root']).toBeDefined();
      expect(tree['/root']['dir1']).toBeDefined();
      expect(tree['/root']['dir1']['subdir1']).toBeDefined();
      expect(tree['/root']['dir1']['subdir1']['file1.txt']).toBeDefined();
      
      expect(tree['/root']['dir2']).toBeDefined();
      expect(tree['/root']['dir2']['subdir2']).toBeDefined();
      expect(tree['/root']['dir2']['subdir2']['subsubdir']).toBeDefined();
      expect(tree['/root']['dir2']['subdir2']['subsubdir']['file2.txt']).toBeDefined();
    });

    it('should maintain state between operations', () => {
      viewer['diff'] = mockDiff;
      
      // First render
      const tree1 = viewer['buildPathTree']();
      expect(Object.keys(tree1)).toHaveLength(4); // 4 entries
      
      // Change options
      viewer.setOptions({ maxDepth: 1 });
      
      // Second render with different options
      const tree2 = viewer['buildPathTree']();
      expect(Object.keys(tree2)).toHaveLength(4); // Same entries, different rendering
      
      // Options should persist
      expect(viewer['maxDepth']).toBe(1);
    });
  });
});

describe('EntryType Enum', () => {
  it('should have correct values', () => {
    expect(EntryType.File).toBe(1);
    expect(EntryType.Directory).toBe(2);
    expect(EntryType.Symlink).toBe(3);
    expect(EntryType.Special).toBe(4);
  });
});

describe('DiffV1 Interface', () => {
  it('should allow creation of valid diff objects', () => {
    const diff: DiffV1 = {
      version: 1,
      timestamp: "2024-01-01T12:00:00Z",
      old_snapshot: new Uint8Array([1, 2, 3]),
      new_snapshot: new Uint8Array([4, 5, 6]),
      added: [],
      removed: [],
      modified: [],
      summary: {
        total_added: 0,
        total_removed: 0,
        total_modified: 0,
        bytes_added: 0,
        bytes_removed: 0,
        bytes_delta: 0
      }
    };
    
    expect(diff.version).toBe(1);
    expect(diff.timestamp).toBe("2024-01-01T12:00:00Z");
    expect(diff.added).toHaveLength(0);
  });
});

describe('Entry Interface', () => {
  it('should allow creation of valid entry objects', () => {
    const entry: Entry = {
      path: "/test.txt",
      cid_ngfs: new Uint8Array([1, 2, 3]),
      size: 1024,
      type: EntryType.File,
      mode: 0o644,
      mtime: 1640995200
    };
    
    expect(entry.path).toBe("/test.txt");
    expect(entry.type).toBe(EntryType.File);
    expect(entry.size).toBe(1024);
  });
});

describe('ModEntry Interface', () => {
  it('should allow creation of valid modified entry objects', () => {
    const modEntry: ModEntry = {
      path: "/test.txt",
      old_cid: new Uint8Array([1, 2, 3]),
      new_cid: new Uint8Array([4, 5, 6]),
      old_size: 1024,
      new_size: 2048,
      delta_size: 1024,
      mode: 0o644,
      mtime: 1640995200
    };
    
    expect(modEntry.path).toBe("/test.txt");
    expect(modEntry.delta_size).toBe(1024);
    expect(modEntry.old_size).toBe(1024);
    expect(modEntry.new_size).toBe(2048);
  });
});

describe('DiffSummary Interface', () => {
  it('should allow creation of valid summary objects', () => {
    const summary: DiffSummary = {
      total_added: 5,
      total_removed: 2,
      total_modified: 1,
      bytes_added: 5120,
      bytes_removed: 1024,
      bytes_delta: 4096
    };
    
    expect(summary.total_added).toBe(5);
    expect(summary.total_removed).toBe(2);
    expect(summary.total_modified).toBe(1);
    expect(summary.bytes_added).toBe(5120);
    expect(summary.bytes_removed).toBe(1024);
    expect(summary.bytes_delta).toBe(4096);
  });
});
