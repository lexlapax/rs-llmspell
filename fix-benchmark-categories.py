#!/usr/bin/env python3
"""Fix benchmark category placement in benchmark files."""

import re
from pathlib import Path

def fix_benchmark_category(file_path):
    """Fix benchmark category placement in a file."""
    content = file_path.read_text()
    
    # Remove wrongly placed attributes
    lines = content.split('\n')
    new_lines = []
    
    for line in lines:
        # Skip lines that have the attribute in wrong places
        if '#[cfg_attr(test_category = "benchmark")]' in line:
            continue
        new_lines.append(line)
    
    # Add the attribute at the module level (top of file)
    # Find where to insert after initial comments
    insert_index = 0
    for i, line in enumerate(new_lines):
        # Skip ABOUTME comments and empty lines at the beginning
        if line.strip().startswith('//') or line.strip() == '':
            insert_index = i + 1
        else:
            break
    
    # Insert the attribute
    new_lines.insert(insert_index, '#![cfg_attr(test_category = "benchmark")]')
    new_lines.insert(insert_index + 1, '')
    
    file_path.write_text('\n'.join(new_lines))
    return True

def main():
    """Fix benchmark categories in all benchmark files."""
    root = Path("/Users/spuri/projects/lexlapax/rs-llmspell")
    
    # Find all benchmark files
    benchmark_files = list(root.glob("**/benches/*.rs"))
    
    print(f"Found {len(benchmark_files)} benchmark files to fix")
    
    for bench_file in benchmark_files:
        relative_path = bench_file.relative_to(root)
        if fix_benchmark_category(bench_file):
            print(f"Fixed: {relative_path}")
    
    print(f"\nAll {len(benchmark_files)} benchmark files fixed")

if __name__ == "__main__":
    main()