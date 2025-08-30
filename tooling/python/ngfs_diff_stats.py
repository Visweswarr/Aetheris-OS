#!/usr/bin/env python3
"""
NGFS Diff Statistics Analyzer - Analyze and merge NGFS snapshot diffs
"""

import argparse
import json
import os
import sys
from datetime import datetime
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
import cbor2

@dataclass
class DiffStats:
    """Statistics for a single diff"""
    timestamp: str
    added: int
    removed: int
    modified: int
    bytes_added: int
    bytes_removed: int
    bytes_delta: int
    total_changes: int

@dataclass
class TimelineEntry:
    """Entry in the diff timeline"""
    timestamp: str
    diff_id: str
    changes: DiffStats
    description: Optional[str] = None

class NgfsDiffAnalyzer:
    """Analyzer for NGFS diff files"""
    
    def __init__(self):
        self.diffs: List[Dict[str, Any]] = []
        self.timeline: List[TimelineEntry] = []
    
    def load_diff(self, file_path: str) -> bool:
        """Load a diff file (CBOR or JSON)"""
        try:
            if file_path.endswith('.cbor'):
                with open(file_path, 'rb') as f:
                    diff_data = cbor2.load(f)
            elif file_path.endswith('.json'):
                with open(file_path, 'r') as f:
                    diff_data = json.load(f)
            else:
                print(f"Error: Unsupported file format: {file_path}")
                return False
            
            # Validate diff structure
            if not self.validate_diff_structure(diff_data):
                print(f"Error: Invalid diff structure in {file_path}")
                return False
            
            self.diffs.append(diff_data)
            return True
            
        except Exception as e:
            print(f"Error loading {file_path}: {e}")
            return False
    
    def validate_diff_structure(self, diff_data: Dict[str, Any]) -> bool:
        """Validate that the diff has the expected structure"""
        required_fields = ['version', 'timestamp', 'old_snapshot', 'new_snapshot', 
                          'added', 'removed', 'modified', 'summary']
        
        for field in required_fields:
            if field not in diff_data:
                print(f"Missing required field: {field}")
                return False
        
        # Check summary fields
        summary = diff_data['summary']
        summary_fields = ['total_added', 'total_removed', 'total_modified', 
                         'bytes_added', 'bytes_removed', 'bytes_delta']
        
        for field in summary_fields:
            if field not in summary:
                print(f"Missing summary field: {field}")
                return False
        
        return True
    
    def analyze_diff(self, diff_data: Dict[str, Any]) -> DiffStats:
        """Analyze a single diff and extract statistics"""
        summary = diff_data['summary']
        
        return DiffStats(
            timestamp=diff_data['timestamp'],
            added=summary['total_added'],
            removed=summary['total_removed'],
            modified=summary['total_modified'],
            bytes_added=summary['bytes_added'],
            bytes_removed=summary['bytes_removed'],
            bytes_delta=summary['bytes_delta'],
            total_changes=summary['total_added'] + summary['total_removed'] + summary['total_modified']
        )
    
    def build_timeline(self) -> None:
        """Build a timeline from loaded diffs"""
        self.timeline.clear()
        
        for i, diff in enumerate(self.diffs):
            stats = self.analyze_diff(diff)
            
            # Generate a diff ID if not present
            diff_id = f"diff_{i:03d}"
            if 'diff_id' in diff:
                diff_id = diff['diff_id']
            
            # Extract description if available
            description = None
            if 'description' in diff:
                description = diff['description']
            
            entry = TimelineEntry(
                timestamp=stats.timestamp,
                diff_id=diff_id,
                changes=stats,
                description=description
            )
            
            self.timeline.append(entry)
        
        # Sort timeline by timestamp
        self.timeline.sort(key=lambda x: x.timestamp)
    
    def print_summary_table(self, diff_data: Dict[str, Any]) -> None:
        """Print a summary table for a single diff"""
        stats = self.analyze_diff(diff_data)
        
        print("NGFS Diff Summary")
        print("=================")
        print(f"Timestamp:     {stats.timestamp}")
        print(f"Added:         {stats.added:>6} entries")
        print(f"Removed:       {stats.removed:>6} entries")
        print(f"Modified:      {stats.modified:>6} entries")
        print(f"Total Changes: {stats.total_changes:>6}")
        print()
        print("Size Changes:")
        print(f"  Added:       {self.format_bytes(stats.bytes_added):>10}")
        print(f"  Removed:     {self.format_bytes(stats.bytes_removed):>10}")
        print(f"  Net Delta:   {self.format_bytes(stats.bytes_delta):>10}")
        print()
        
        # Print detailed entries if available
        if diff_data['added']:
            print("Added Files:")
            for entry in diff_data['added']:
                size_str = self.format_bytes(entry.get('size', 0))
                print(f"  + {entry['path']:<30} {size_str:>10}")
            print()
        
        if diff_data['removed']:
            print("Removed Files:")
            for entry in diff_data['removed']:
                size_str = self.format_bytes(entry.get('size', 0))
                print(f"  - {entry['path']:<30} {size_str:>10}")
            print()
        
        if diff_data['modified']:
            print("Modified Files:")
            for entry in diff_data['modified']:
                delta_str = self.format_bytes(entry.get('delta_size', 0))
                print(f"  ~ {entry['path']:<30} {delta_str:>10}")
            print()
    
    def print_timeline_summary(self) -> None:
        """Print a summary of the diff timeline"""
        if not self.timeline:
            print("No diffs loaded")
            return
        
        print("NGFS Diff Timeline")
        print("==================")
        print()
        
        total_added = sum(entry.changes.added for entry in self.timeline)
        total_removed = sum(entry.changes.removed for entry in self.timeline)
        total_modified = sum(entry.changes.modified for entry in self.timeline)
        total_bytes_added = sum(entry.changes.bytes_added for entry in self.timeline)
        total_bytes_removed = sum(entry.changes.bytes_removed for entry in self.timeline)
        net_bytes_delta = sum(entry.changes.bytes_delta for entry in self.timeline)
        
        print(f"Total Diffs:       {len(self.timeline):>6}")
        print(f"Total Added:       {total_added:>6} entries")
        print(f"Total Removed:     {total_removed:>6} entries")
        print(f"Total Modified:    {total_modified:>6} entries")
        print(f"Total Changes:     {total_added + total_removed + total_modified:>6}")
        print()
        print("Cumulative Size Changes:")
        print(f"  Total Added:     {self.format_bytes(total_bytes_added):>10}")
        print(f"  Total Removed:   {self.format_bytes(total_bytes_removed):>10}")
        print(f"  Net Delta:       {self.format_bytes(net_bytes_delta):>10}")
        print()
        
        # Print timeline entries
        print("Timeline:")
        for entry in self.timeline:
            date_str = entry.timestamp[:19] if len(entry.timestamp) > 19 else entry.timestamp
            desc_str = f" - {entry.description}" if entry.description else ""
            print(f"  {date_str} [{entry.diff_id}] +{entry.changes.added}/-{entry.changes.removed}/~{entry.changes.modified}{desc_str}")
    
    def merge_diffs(self, output_file: str, format_type: str = 'cbor') -> bool:
        """Merge multiple diffs into a single file"""
        if not self.diffs:
            print("Error: No diffs to merge")
            return False
        
        try:
            merged_data = {
                'merged_timestamp': datetime.utcnow().isoformat() + 'Z',
                'total_diffs': len(self.diffs),
                'diffs': self.diffs,
                'summary': self.calculate_merged_summary()
            }
            
            if format_type == 'json':
                with open(output_file, 'w') as f:
                    json.dump(merged_data, f, indent=2)
            elif format_type == 'cbor':
                with open(output_file, 'wb') as f:
                    cbor2.dump(merged_data, f)
            else:
                print(f"Error: Unsupported output format: {format_type}")
                return False
            
            print(f"Successfully merged {len(self.diffs)} diffs to {output_file}")
            return True
            
        except Exception as e:
            print(f"Error merging diffs: {e}")
            return False
    
    def calculate_merged_summary(self) -> Dict[str, Any]:
        """Calculate summary statistics across all diffs"""
        total_added = sum(diff['summary']['total_added'] for diff in self.diffs)
        total_removed = sum(diff['summary']['total_removed'] for diff in self.diffs)
        total_modified = sum(diff['summary']['total_modified'] for diff in self.diffs)
        total_bytes_added = sum(diff['summary']['bytes_added'] for diff in self.diffs)
        total_bytes_removed = sum(diff['summary']['bytes_removed'] for diff in self.diffs)
        net_bytes_delta = sum(diff['summary']['bytes_delta'] for diff in self.diffs)
        
        return {
            'total_added': total_added,
            'total_removed': total_removed,
            'total_modified': total_modified,
            'bytes_added': total_bytes_added,
            'bytes_removed': total_bytes_removed,
            'bytes_delta': net_bytes_delta,
            'total_changes': total_added + total_removed + total_modified
        }
    
    def export_csv(self, output_file: str) -> bool:
        """Export diff timeline to CSV format"""
        if not self.timeline:
            print("Error: No timeline to export")
            return False
        
        try:
            with open(output_file, 'w') as f:
                # Write header
                f.write("Timestamp,DiffID,Added,Removed,Modified,TotalChanges,BytesAdded,BytesRemoved,BytesDelta,Description\n")
                
                # Write data rows
                for entry in self.timeline:
                    desc = entry.description.replace('"', '""') if entry.description else ""
                    f.write(f'"{entry.timestamp}","{entry.diff_id}",{entry.changes.added},{entry.changes.removed},{entry.changes.modified},{entry.changes.total_changes},{entry.changes.bytes_added},{entry.changes.bytes_removed},{entry.changes.bytes_delta},"{desc}"\n')
            
            print(f"Successfully exported timeline to {output_file}")
            return True
            
        except Exception as e:
            print(f"Error exporting CSV: {e}")
            return False
    
    def format_bytes(self, bytes_value: int) -> str:
        """Format bytes in human-readable format"""
        if bytes_value == 0:
            return "0 B"
        
        units = ['B', 'KB', 'MB', 'GB', 'TB']
        unit_index = 0
        value = float(bytes_value)
        
        while value >= 1024.0 and unit_index < len(units) - 1:
            value /= 1024.0
            unit_index += 1
        
        if unit_index == 0:
            return f"{int(value)} {units[unit_index]}"
        else:
            return f"{value:.1f} {units[unit_index]}"
    
    def clear(self) -> None:
        """Clear all loaded diffs and timeline"""
        self.diffs.clear()
        self.timeline.clear()

def main():
    parser = argparse.ArgumentParser(
        description="NGFS Diff Statistics Analyzer",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Analyze a single diff file
  ngfs_diff_stats.py --diff diff.cbor
  
  # Load multiple diffs and show timeline
  ngfs_diff_stats.py --diff diff1.cbor --diff diff2.cbor --timeline
  
  # Merge diffs and export
  ngfs_diff_stats.py --diff diff1.cbor --diff diff2.cbor --merge merged.cbor
  
  # Export timeline to CSV
  ngfs_diff_stats.py --diff diff1.cbor --diff diff2.cbor --export-csv timeline.csv
        """
    )
    
    parser.add_argument('--diff', '-d', action='append', required=True,
                       help='Diff file to analyze (can specify multiple)')
    parser.add_argument('--timeline', '-t', action='store_true',
                       help='Show timeline summary')
    parser.add_argument('--merge', '-m', metavar='OUTPUT',
                       help='Merge diffs to output file')
    parser.add_argument('--export-csv', metavar='OUTPUT',
                       help='Export timeline to CSV file')
    parser.add_argument('--format', choices=['cbor', 'json'], default='cbor',
                       help='Output format for merged file (default: cbor)')
    parser.add_argument('--verbose', '-v', action='store_true',
                       help='Verbose output')
    
    args = parser.parse_args()
    
    analyzer = NgfsDiffAnalyzer()
    
    # Load all diff files
    for diff_file in args.diff:
        if not os.path.exists(diff_file):
            print(f"Error: Diff file not found: {diff_file}")
            sys.exit(1)
        
        if args.verbose:
            print(f"Loading diff file: {diff_file}")
        
        if not analyzer.load_diff(diff_file):
            print(f"Failed to load diff file: {diff_file}")
            sys.exit(1)
    
    # Build timeline
    analyzer.build_timeline()
    
    # Show individual diff summaries
    for i, diff_file in enumerate(args.diff):
        print(f"\n{'='*60}")
        print(f"Diff File {i+1}: {diff_file}")
        print(f"{'='*60}")
        analyzer.print_summary_table(analyzer.diffs[i])
    
    # Show timeline if requested
    if args.timeline:
        print(f"\n{'='*60}")
        analyzer.print_timeline_summary()
    
    # Merge diffs if requested
    if args.merge:
        if not analyzer.merge_diffs(args.merge, args.format):
            sys.exit(1)
    
    # Export CSV if requested
    if args.export_csv:
        if not analyzer.export_csv(args.export_csv):
            sys.exit(1)
    
    print("\nAnalysis complete!")

if __name__ == "__main__":
    main()
