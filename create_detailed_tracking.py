#!/usr/bin/env python3
import re
from collections import defaultdict
import json

def categorize_warning(warning_msg):
    """Categorize warning messages into types."""
    if 'early dropped' in warning_msg or 'temporary with significant' in warning_msg:
        return 'early_drop'
    elif 'identical body' in warning_msg or 'match arm' in warning_msg and 'identical' in warning_msg:
        return 'identical_match_arms'
    elif 'Clone::clone()' in warning_msg or 'inefficient' in warning_msg and 'clone' in warning_msg:
        return 'inefficient_clone'
    elif 'format!' in warning_msg or 'variables can be used directly' in warning_msg:
        return 'format_string'
    elif 'unused `async`' in warning_msg:
        return 'unused_async'
    elif 'map_or' in warning_msg or 'unwrap_or' in warning_msg or 'map(<f>).unwrap_or' in warning_msg:
        return 'option_result_patterns'
    elif 'unnecessarily wrapped by `Result`' in warning_msg:
        return 'unnecessary_result'
    elif 'passed by value, but not consumed' in warning_msg or 'pass' in warning_msg and 'value' in warning_msg:
        return 'pass_by_value'
    elif 'Default::default()' in warning_msg or 'default()` is more clear' in warning_msg:
        return 'default_trait'
    elif 'panic' in warning_msg:
        return 'panic_issues'
    elif 'redundant' in warning_msg:
        return 'redundant_code'
    elif 'cognitive complexity' in warning_msg:
        return 'cognitive_complexity'
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
    elif 'cast' in warning_msg or 'casting' in warning_msg:
        return 'cast_issues'
    elif 'bool' in warning_msg and 'complex' in warning_msg:
        return 'bool_logic'
    elif 'needless' in warning_msg:
        return 'needless_patterns'
    elif 'unnecessary' in warning_msg:
        return 'unnecessary_code'
    else:
        return 'other'

# Parse the clippy output file
with open('/Users/spuri/projects/lexlapax/rs-llmspell/full_clippy_output.txt', 'r') as f:
    lines = f.readlines()

# Dictionary to store warnings with full details
warnings_by_category = defaultdict(lambda: defaultdict(list))
warnings_by_crate = defaultdict(lambda: defaultdict(list))

i = 0
while i < len(lines):
    line = lines[i].strip()
    
    # Look for warning lines
    if line.startswith('warning:'):
        warning_msg = line[8:].strip()
        
        # Look for location in next lines
        location = None
        suggestion = None
        j = i + 1
        
        # Find the location line (with -->)
        while j < len(lines) and j < i + 10:
            if '-->' in lines[j]:
                location_match = re.match(r'\s*-->\s+([^:]+):(\d+):(\d+)', lines[j])
                if location_match:
                    file_path = location_match.group(1)
                    line_num = location_match.group(2)
                    col_num = location_match.group(3)
                    location = f"{file_path}:{line_num}:{col_num}"
                break
            j += 1
        
        # Look for help/suggestion
        j = i + 1
        while j < len(lines) and j < i + 20:
            if lines[j].strip().startswith('= help:'):
                suggestion = lines[j].strip()[7:].strip()
                break
            elif lines[j].strip().startswith('help:'):
                suggestion = lines[j].strip()[5:].strip()
                break
            j += 1
        
        if location:
            # Extract crate name from path
            crate_name = 'unknown'
            if 'llmspell-' in file_path:
                crate_match = re.search(r'(llmspell-[^/]+)', file_path)
                if crate_match:
                    crate_name = crate_match.group(1)
            
            # Categorize the warning
            category = categorize_warning(warning_msg)
            
            warning_entry = {
                'location': location,
                'message': warning_msg,
                'suggestion': suggestion,
                'crate': crate_name
            }
            
            warnings_by_category[category][crate_name].append(warning_entry)
            warnings_by_crate[crate_name][category].append(warning_entry)
    
    i += 1

# Write detailed tracking file
with open('/Users/spuri/projects/lexlapax/rs-llmspell/detailed_tracking.txt', 'w') as f:
    f.write("# Phase 10.7 Detailed Warning Tracking File\n")
    f.write("# Generated from full workspace clippy scan\n")
    f.write("# Each warning includes exact file location for efficient fixing\n\n")
    
    # Summary statistics
    total_warnings = sum(len(warnings) for cat in warnings_by_category.values() for warnings in cat.values())
    f.write(f"## TOTAL WARNINGS: {total_warnings}\n\n")
    
    f.write("## WARNINGS BY CATEGORY (Priority Order)\n\n")
    
    # Priority order based on impact
    priority_categories = [
        ('early_drop', 'Performance - Temporary with significant Drop can be optimized'),
        ('identical_match_arms', 'Code Quality - Match arms with identical bodies'),
        ('option_result_patterns', 'Idiomatic - Better use of Option/Result combinators'),
        ('pass_by_value', 'Performance - Arguments passed by value but not consumed'),
        ('default_trait', 'Style - Using Default::default() instead of Type::default()'),
        ('panic_issues', 'Error Handling - Panic message improvements'),
        ('format_string', 'Style - Variables can be directly interpolated'),
        ('redundant_code', 'Code Quality - Redundant closures and patterns'),
        ('unnecessary_result', 'API Design - Functions that always return Ok'),
        ('inefficient_clone', 'Performance - Use clone_from() instead of assignment'),
        ('cast_issues', 'Type Safety - Unnecessary or improper casts'),
        ('cognitive_complexity', 'Maintainability - Functions that are too complex'),
        ('unnecessary_deref', 'Style - Unnecessary reference/dereference'),
        ('wildcard_patterns', 'Code Quality - Wildcard pattern issues'),
        ('unused_async', 'Performance - Async functions without await'),
        ('bool_logic', 'Logic - Complex boolean expressions'),
        ('float_comparison', 'Correctness - Float comparison issues'),
        ('or_patterns', 'Style - Unnested or-patterns'),
        ('assertion_issues', 'Testing - Assertion improvements'),
        ('needless_patterns', 'Style - Needless code patterns'),
        ('unnecessary_code', 'Code Quality - Generally unnecessary code'),
        ('other', 'Miscellaneous - Various other warnings')
    ]
    
    for category, description in priority_categories:
        if category in warnings_by_category:
            total_in_category = sum(len(warnings) for warnings in warnings_by_category[category].values())
            f.write(f"### {category.upper()} ({total_in_category} warnings)\n")
            f.write(f"Description: {description}\n\n")
            
            for crate in sorted(warnings_by_category[category].keys()):
                warnings = warnings_by_category[category][crate]
                f.write(f"#### {crate} ({len(warnings)} warnings)\n\n")
                
                for i, warning in enumerate(warnings, 1):
                    f.write(f"{i}. {warning['location']}\n")
                    f.write(f"   Message: {warning['message']}\n")
                    if warning['suggestion']:
                        f.write(f"   Suggestion: {warning['suggestion']}\n")
                    f.write("\n")
            f.write("\n")
    
    f.write("\n## WARNINGS BY CRATE\n\n")
    
    for crate in sorted(warnings_by_crate.keys()):
        total_in_crate = sum(len(warnings) for warnings in warnings_by_crate[crate].values())
        f.write(f"### {crate} ({total_in_crate} warnings)\n\n")
        
        for category in sorted(warnings_by_crate[crate].keys()):
            warnings = warnings_by_crate[crate][category]
            f.write(f"#### {category} ({len(warnings)} warnings)\n")
            for warning in warnings:
                f.write(f"- {warning['location']}: {warning['message'][:80]}...\n")
        f.write("\n")

print(f"Created detailed tracking file with {total_warnings} warnings")
print("File: detailed_tracking.txt")