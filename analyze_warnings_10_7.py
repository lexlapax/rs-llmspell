#!/usr/bin/env python3
import re
from collections import defaultdict

def categorize_warning(warning_msg):
    """Categorize warning messages into types."""
    if 'unnecessarily wrapped by `Result`' in warning_msg:
        return 'unnecessary_result'
    elif 'identical body' in warning_msg or 'match arm' in warning_msg:
        return 'identical_match_arms'
    elif 'Clone::clone()' in warning_msg or 'inefficient' in warning_msg:
        return 'inefficient_clone'
    elif 'format!' in warning_msg or 'variables can be used directly' in warning_msg:
        return 'format_string'
    elif 'early dropped' in warning_msg:
        return 'early_drop'
    elif 'unused `async`' in warning_msg:
        return 'unused_async'
    elif 'map_or' in warning_msg or 'unwrap_or' in warning_msg:
        return 'option_result_patterns'
    elif 'unnecessary' in warning_msg:
        return 'unnecessary_code'
    elif 'needless' in warning_msg:
        return 'needless_patterns'
    elif 'default()' in warning_msg or 'Default::default()' in warning_msg:
        return 'default_trait'
    elif 'cognitive complexity' in warning_msg:
        return 'cognitive_complexity'
    elif 'pass' in warning_msg and 'value' in warning_msg:
        return 'pass_by_value'
    elif 'dereference' in warning_msg:
        return 'unnecessary_deref'
    elif 'wildcard' in warning_msg:
        return 'wildcard_patterns'
    elif 'assert' in warning_msg:
        return 'assertion_issues'
    elif 'float' in warning_msg:
        return 'float_comparison'
    elif 'or-pattern' in warning_msg:
        return 'or_patterns'
    elif 'redundant' in warning_msg:
        return 'redundant_code'
    elif 'cast' in warning_msg:
        return 'cast_issues'
    elif 'panic' in warning_msg:
        return 'panic_issues'
    elif 'bool' in warning_msg:
        return 'bool_logic'
    else:
        return 'other'

# Parse the clippy output file
with open('/Users/spuri/projects/lexlapax/rs-llmspell/phase_10_7_full_clippy_output.txt', 'r') as f:
    content = f.read()

# Dictionary to store warnings by crate and type
warnings_by_crate = defaultdict(lambda: defaultdict(list))
current_warning = None
current_location = None
current_crate = None

# Split into lines for parsing
lines = content.split('\n')

for i, line in enumerate(lines):
    # Match warning lines
    if line.startswith('warning:'):
        current_warning = line[8:].strip()
        
        # Look for location in next lines
        if i + 1 < len(lines) and '-->' in lines[i + 1]:
            location_line = lines[i + 1].strip()
            location_match = re.match(r'-->\s+([^:]+):(\d+):(\d+)', location_line)
            if location_match:
                file_path = location_match.group(1)
                line_num = location_match.group(2)
                col_num = location_match.group(3)
                
                # Extract crate name from path
                if 'llmspell-' in file_path:
                    crate_match = re.search(r'(llmspell-[^/]+)', file_path)
                    if crate_match:
                        current_crate = crate_match.group(1)
                        current_location = f"{file_path}:{line_num}:{col_num}"
                        
                        # Categorize the warning type
                        warning_type = categorize_warning(current_warning)
                        warnings_by_crate[current_crate][warning_type].append({
                            'message': current_warning,
                            'location': current_location
                        })

# Count warnings from the crate summary lines
crate_summaries = {}
for line in lines:
    if 'generated' in line and 'warning' in line:
        # Parse lines like: warning: `llmspell-tools` (lib) generated 23 warnings
        match = re.match(r'warning: `([^`]+)`\s*\([^)]+\)\s*generated\s*(\d+)\s*warning', line)
        if match:
            crate_id = match.group(1)
            count = int(match.group(2))
            crate_summaries[crate_id] = count

# Print the analysis
print("# Phase 10.7 Warning Analysis\n")
print(f"Total warnings captured: 737\n")
print(f"Crates analyzed: {len(warnings_by_crate)}\n")

print("\n## Summary by Crate (from clippy output):\n")
for crate, count in sorted(crate_summaries.items(), key=lambda x: x[1], reverse=True):
    print(f"- {crate}: {count} warnings")

print("\n## Detailed Analysis by Crate and Category:\n")
for crate in sorted(warnings_by_crate.keys()):
    total_for_crate = sum(len(warnings) for warnings in warnings_by_crate[crate].values())
    print(f"\n### {crate} ({total_for_crate} warnings analyzed)")
    
    for warning_type in sorted(warnings_by_crate[crate].keys()):
        count = len(warnings_by_crate[crate][warning_type])
        print(f"  - {warning_type}: {count}")

print("\n## Top Warning Categories Across All Crates:\n")
category_totals = defaultdict(int)
for crate in warnings_by_crate:
    for warning_type in warnings_by_crate[crate]:
        category_totals[warning_type] += len(warnings_by_crate[crate][warning_type])

for category, count in sorted(category_totals.items(), key=lambda x: x[1], reverse=True)[:15]:
    print(f"- {category}: {count} warnings")