#!/usr/bin/env python3

import re
import sys

def extract_warnings(file_path):
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Pattern to match warnings with file paths
    unused_self_pattern = r'warning: unused `self` argument\s*\n\s*--> ([^\n]+)'
    too_many_lines_pattern = r'warning: this function has too many lines \((\d+)/(\d+)\)\s*\n\s*--> ([^\n]+)'
    
    unused_self_matches = re.findall(unused_self_pattern, content)
    too_many_lines_matches = re.findall(too_many_lines_pattern, content)
    
    print("# Phase 10.5 Function Refactoring Tracking File")
    print(f"# Total: {len(unused_self_matches) + len(too_many_lines_matches)} warnings ({len(unused_self_matches)} unused_self + {len(too_many_lines_matches)} too_many_lines)")
    print()
    
    print("## UNUSED_SELF WARNINGS")
    print(f"# Count: {len(unused_self_matches)}")
    for match in sorted(set(unused_self_matches)):
        print(f"[ ] {match}")
    
    print()
    print("## TOO_MANY_LINES WARNINGS")
    print(f"# Count: {len(too_many_lines_matches)}")
    for current, limit, file_path in sorted(set(too_many_lines_matches), key=lambda x: x[2]):
        print(f"[ ] {file_path} ({current}/{limit} lines)")
    
    # Summary by file
    print("\n## SUMMARY BY FILE")
    file_counts = {}
    for match in unused_self_matches:
        file_name = match.split(':')[0]
        if file_name not in file_counts:
            file_counts[file_name] = {'unused_self': 0, 'too_many_lines': 0}
        file_counts[file_name]['unused_self'] += 1
    
    for _, _, file_path in too_many_lines_matches:
        file_name = file_path.split(':')[0]
        if file_name not in file_counts:
            file_counts[file_name] = {'unused_self': 0, 'too_many_lines': 0}
        file_counts[file_name]['too_many_lines'] += 1
    
    for file_name in sorted(file_counts.keys()):
        counts = file_counts[file_name]
        total = counts['unused_self'] + counts['too_many_lines']
        print(f"{file_name}: {total} warnings (unused_self: {counts['unused_self']}, too_many_lines: {counts['too_many_lines']})")

if __name__ == "__main__":
    extract_warnings("/Users/spuri/projects/lexlapax/rs-llmspell/all_clippy_output.txt")