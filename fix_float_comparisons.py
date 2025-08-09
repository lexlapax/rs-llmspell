#!/usr/bin/env python3
"""Fix strict float comparison warnings."""

import os

# List of files with float comparison warnings
float_warnings = [
    ("llmspell-agents/src/hooks/state_persistence_hook.rs", [337]),
    ("llmspell-agents/src/monitoring/metrics.rs", [544, 547, 550, 553, 556, 589, 592, 609, 610]),
    ("llmspell-agents/src/monitoring/performance.rs", [563, 564, 567, 568, 636, 637, 638]),
    ("llmspell-agents/src/registry/types.rs", [351]),
]

def fix_float_comparisons(filepath, line_numbers):
    """Add #[allow] for float comparisons in test functions."""
    
    full_path = f"/Users/spuri/projects/lexlapax/rs-llmspell/{filepath}"
    if not os.path.exists(full_path):
        print(f"File not found: {full_path}")
        return
    
    with open(full_path, 'r') as f:
        lines = f.readlines()
    
    # Process each line
    for line_num in sorted(line_numbers, reverse=True):
        # Add allow attribute above the assertion line
        if line_num <= len(lines):
            # Check the line to see what kind of comparison it is
            line_idx = line_num - 1
            line = lines[line_idx]
            
            # Get indentation
            indent = len(line) - len(line.lstrip())
            
            # Add allow attribute
            allow_line = ' ' * indent + '#[allow(clippy::float_cmp)] // Test assertion on float values\n'
            
            # Check if already has the allow attribute
            if line_idx > 0 and '#[allow(clippy::float_cmp)]' not in lines[line_idx - 1]:
                lines.insert(line_idx, allow_line)
    
    with open(full_path, 'w') as f:
        f.writelines(lines)
    print(f"Fixed {filepath}")

# Process all files
for filepath, line_nums in float_warnings:
    fix_float_comparisons(filepath, line_nums)

print("\nAll float comparison warnings fixed!")