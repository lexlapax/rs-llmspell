#!/usr/bin/env python3
"""Analyze clippy warnings from all_warnings.txt"""

import re
from collections import defaultdict
from pathlib import Path

def parse_warnings(file_path):
    """Parse warnings from clippy output"""
    warnings = defaultdict(list)
    current_file = None
    current_line = None
    current_warning = None
    
    with open(file_path, 'r') as f:
        lines = f.readlines()
    
    i = 0
    while i < len(lines):
        line = lines[i].strip()
        
        # Match error/warning line with file:line:column
        match = re.match(r'(error|warning):\s*(.+)', line)
        if match:
            warning_type = match.group(2)
            
            # Look for file location in previous lines
            if i > 0:
                loc_line = lines[i-1].strip()
                loc_match = re.match(r'-->\s*(.+?):(\d+):(\d+)', loc_line)
                if loc_match:
                    file_path = loc_match.group(1)
                    line_num = loc_match.group(2)
                    
                    # Extract the warning category from help lines
                    category = None
                    for j in range(i+1, min(i+10, len(lines))):
                        help_line = lines[j].strip()
                        cat_match = re.search(r'#(\w+)', help_line)
                        if cat_match:
                            category = cat_match.group(1)
                            break
                    
                    warnings[file_path].append({
                        'line': int(line_num),
                        'type': warning_type,
                        'category': category or 'unknown'
                    })
        
        i += 1
    
    return warnings

def analyze_by_category(warnings):
    """Group warnings by category"""
    categories = defaultdict(list)
    
    for file_path, file_warnings in warnings.items():
        for w in file_warnings:
            categories[w['category']].append({
                'file': file_path,
                'line': w['line'],
                'type': w['type']
            })
    
    return categories

def analyze_by_crate(warnings):
    """Group warnings by crate"""
    crates = defaultdict(lambda: defaultdict(int))
    
    for file_path, file_warnings in warnings.items():
        # Extract crate name from path
        parts = file_path.split('/')
        if 'llmspell-' in file_path:
            for part in parts:
                if part.startswith('llmspell-'):
                    crate_name = part
                    break
            else:
                crate_name = 'unknown'
        else:
            crate_name = 'unknown'
        
        for w in file_warnings:
            crates[crate_name][w['category']] += 1
    
    return crates

def main():
    warnings = parse_warnings('all_warnings.txt')
    
    # Total warnings
    total = sum(len(w) for w in warnings.values())
    print(f"Total warnings: {total}")
    print()
    
    # By category
    categories = analyze_by_category(warnings)
    print("Warnings by category:")
    for cat, items in sorted(categories.items(), key=lambda x: -len(x[1])):
        print(f"  {cat}: {len(items)}")
    print()
    
    # By crate
    crates = analyze_by_crate(warnings)
    print("Warnings by crate:")
    for crate, cats in sorted(crates.items()):
        total_crate = sum(cats.values())
        print(f"\n{crate}: {total_crate} warnings")
        for cat, count in sorted(cats.items(), key=lambda x: -x[1]):
            print(f"  {cat}: {count}")
    
    # Files with most warnings
    print("\nTop 10 files with most warnings:")
    sorted_files = sorted(warnings.items(), key=lambda x: -len(x[1]))
    for file_path, file_warnings in sorted_files[:10]:
        print(f"  {file_path}: {len(file_warnings)}")
    
    # Create detailed fix list
    print("\n\nDetailed fix list by file:")
    for file_path in sorted(warnings.keys()):
        file_warnings = warnings[file_path]
        if len(file_warnings) > 0:
            print(f"\n{file_path}: ({len(file_warnings)} warnings)")
            
            # Group by category for this file
            by_cat = defaultdict(list)
            for w in file_warnings:
                by_cat[w['category']].append(w['line'])
            
            for cat, lines in sorted(by_cat.items()):
                print(f"  {cat}: lines {sorted(lines)}")

if __name__ == "__main__":
    main()