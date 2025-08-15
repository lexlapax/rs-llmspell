#!/usr/bin/env python3
"""Parse and analyze clippy warnings from all_warnings.txt"""

import re
from collections import defaultdict

def parse_warnings():
    """Parse warnings from all_warnings.txt"""
    warnings = []
    
    with open('all_warnings.txt', 'r') as f:
        content = f.read()
    
    # Pattern to match warning/error blocks
    pattern = r'(error|warning):\s*(.+?)\n\s*-->\s*(.+?):(\d+):(\d+)'
    matches = re.findall(pattern, content, re.MULTILINE)
    
    for match in matches:
        level, message, file_path, line, col = match
        
        # Extract warning type from message
        warning_type = message.split('\n')[0].strip()
        
        # Try to extract clippy category
        category_match = re.search(r'#(\w+)', content[content.find(message):content.find(message)+500])
        category = category_match.group(1) if category_match else 'unknown'
        
        warnings.append({
            'level': level,
            'type': warning_type,
            'file': file_path,
            'line': int(line),
            'col': int(col),
            'category': category
        })
    
    return warnings

def analyze_warnings(warnings):
    """Analyze and categorize warnings"""
    
    # Group by category
    by_category = defaultdict(list)
    for w in warnings:
        by_category[w['category']].append(w)
    
    # Group by crate
    by_crate = defaultdict(list)
    for w in warnings:
        # Extract crate name
        if 'llmspell-' in w['file']:
            parts = w['file'].split('/')
            for part in parts:
                if part.startswith('llmspell-'):
                    crate = part
                    break
            else:
                crate = 'unknown'
        else:
            crate = 'unknown'
        by_crate[crate].append(w)
    
    # Group by file
    by_file = defaultdict(list)
    for w in warnings:
        by_file[w['file']].append(w)
    
    return by_category, by_crate, by_file

def main():
    warnings = parse_warnings()
    print(f"Total warnings found: {len(warnings)}\n")
    
    by_category, by_crate, by_file = analyze_warnings(warnings)
    
    # Print category summary
    print("=== WARNINGS BY CATEGORY ===")
    for cat in sorted(by_category.keys(), key=lambda x: -len(by_category[x])):
        count = len(by_category[cat])
        print(f"{cat:40} : {count:3}")
    
    print("\n=== WARNINGS BY CRATE ===")
    for crate in sorted(by_crate.keys()):
        warnings_in_crate = by_crate[crate]
        print(f"\n{crate}: {len(warnings_in_crate)} warnings")
        
        # Category breakdown for this crate
        crate_cats = defaultdict(int)
        for w in warnings_in_crate:
            crate_cats[w['category']] += 1
        
        for cat, count in sorted(crate_cats.items(), key=lambda x: -x[1]):
            print(f"  {cat:35} : {count:3}")
    
    print("\n=== TOP 10 FILES WITH MOST WARNINGS ===")
    sorted_files = sorted(by_file.items(), key=lambda x: -len(x[1]))
    for file_path, file_warnings in sorted_files[:10]:
        print(f"{len(file_warnings):3} : {file_path}")
    
    # Create fix priority list
    print("\n=== FIX PRIORITY (by impact) ===")
    print("\n1. Quick wins (automated):")
    print(f"   doc_markdown: {len(by_category.get('doc_markdown', []))}")
    print(f"   redundant_closure_for_method_calls: {len(by_category.get('redundant_closure_for_method_calls', []))}")
    print(f"   bool_to_int_with_if: {len(by_category.get('bool_to_int_with_if', []))}")
    
    print("\n2. Must-use attributes:")
    print(f"   must_use_candidate: {len(by_category.get('must_use_candidate', []))}")
    print(f"   return_self_not_must_use: {len(by_category.get('return_self_not_must_use', []))}")
    
    print("\n3. Match consolidation:")
    print(f"   match_same_arms: {len(by_category.get('match_same_arms', []))}")
    
    print("\n4. Other improvements:")
    print(f"   assigning_clones: {len(by_category.get('assigning_clones', []))}")
    print(f"   map_unwrap_or: {len(by_category.get('map_unwrap_or', []))}")

if __name__ == "__main__":
    main()