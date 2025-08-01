#!/usr/bin/env python3
"""Add benchmark category to all benchmark files."""

import re
from pathlib import Path

def add_benchmark_category(file_path):
    """Add benchmark category to a file if it doesn't already have it."""
    content = file_path.read_text()
    
    # Check if already has test_category
    if 'test_category = "benchmark"' in content:
        return False
    
    # For Criterion benchmarks, we need to add the attribute to the module level
    # Look for criterion_group! or criterion_main! macros
    lines = content.split('\n')
    modified = False
    new_lines = []
    
    # Add at the top of the file after initial comments and use statements
    added_attribute = False
    for i, line in enumerate(lines):
        # Skip initial comments and blank lines
        if i < len(lines) - 1 and not added_attribute:
            # Look for the first non-comment, non-use, non-blank line
            if (line.strip() and 
                not line.strip().startswith('//') and 
                not line.strip().startswith('use ') and
                not line.strip().startswith('#![') and
                not line.strip().startswith('#[')):
                # Add the attribute before this line
                new_lines.append('#[cfg_attr(test_category = "benchmark")]')
                new_lines.append(line)
                added_attribute = True
                modified = True
            else:
                new_lines.append(line)
        else:
            new_lines.append(line)
    
    # If we didn't add it yet (all lines were comments/use/blank), add at end
    if not added_attribute and len(lines) > 0:
        # Find the last use statement or module doc comment
        insert_index = 0
        for i, line in enumerate(lines):
            if line.strip().startswith('use '):
                insert_index = i + 1
            elif line.strip() == '' and i > 0:
                # Found first blank line after uses
                if insert_index > 0:
                    break
        
        if insert_index > 0:
            lines.insert(insert_index, '#[cfg_attr(test_category = "benchmark")]')
            modified = True
            new_lines = lines
    
    if modified:
        file_path.write_text('\n'.join(new_lines))
    
    return modified

def main():
    """Add benchmark categories to all benchmark files."""
    root = Path("/Users/spuri/projects/lexlapax/rs-llmspell")
    
    # Find all benchmark files
    benchmark_files = list(root.glob("**/benches/*.rs"))
    
    print(f"Found {len(benchmark_files)} benchmark files")
    
    updated_count = 0
    
    for bench_file in benchmark_files:
        relative_path = bench_file.relative_to(root)
        if add_benchmark_category(bench_file):
            print(f"Updated: {relative_path}")
            updated_count += 1
        else:
            print(f"Already categorized: {relative_path}")
    
    print(f"\nTotal files updated: {updated_count}")
    print(f"Total benchmark files: {len(benchmark_files)}")

if __name__ == "__main__":
    main()