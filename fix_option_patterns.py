#!/usr/bin/env python3
"""Extract Option/Result pattern warnings from tracking file for systematic fixing"""

import re

tracking_file = "/Users/spuri/projects/lexlapax/rs-llmspell/phase_10_7_detailed_tracking.txt"

# Read tracking file
with open(tracking_file, 'r') as f:
    content = f.read()

# Extract map_or_else warnings
map_or_else_pattern = r'(\d+)\. ([^:]+):(\d+):(\d+)\n   Message: use Option::map_or_else instead of an if let/else'
map_or_else_warnings = re.findall(map_or_else_pattern, content)

# Extract map_or warnings
map_or_pattern = r'(\d+)\. ([^:]+):(\d+):(\d+)\n   Message: use Option::map_or instead of an if let/else'
map_or_warnings = re.findall(map_or_pattern, content)

# Extract unnecessary Result warnings
result_pattern = r'(\d+)\. ([^:]+):(\d+):(\d+)\n   Message: this function\'s return value is unnecessarily wrapped by `Result`'
result_warnings = re.findall(result_pattern, content)

print("Option::map_or_else warnings (31 total):")
print("=" * 60)
for num, file, line, col in map_or_else_warnings[:10]:
    print(f"{file}:{line}:{col}")

print("\nOption::map_or warnings (9 total):")
print("=" * 60)
for num, file, line, col in map_or_warnings:
    print(f"{file}:{line}:{col}")

print("\nUnnecessary Result wrappings (13 total):")
print("=" * 60)
for num, file, line, col in result_warnings:
    print(f"{file}:{line}:{col}")

print(f"\nTotal Option/Result warnings: {len(map_or_else_warnings) + len(map_or_warnings) + len(result_warnings)}")

# Group by file
files_to_fix = {}
for _, file, line, col in map_or_else_warnings:
    if file not in files_to_fix:
        files_to_fix[file] = []
    files_to_fix[file].append((int(line), "map_or_else"))

for _, file, line, col in map_or_warnings:
    if file not in files_to_fix:
        files_to_fix[file] = []
    files_to_fix[file].append((int(line), "map_or"))

for _, file, line, col in result_warnings:
    if file not in files_to_fix:
        files_to_fix[file] = []
    files_to_fix[file].append((int(line), "unnecessary_result"))

print("\n\nFiles to fix (sorted by warning count):")
print("=" * 60)
sorted_files = sorted(files_to_fix.items(), key=lambda x: len(x[1]), reverse=True)
for file, warnings in sorted_files[:15]:
    print(f"{file}: {len(warnings)} warnings")
    for line, warning_type in sorted(warnings)[:3]:
        print(f"  Line {line}: {warning_type}")