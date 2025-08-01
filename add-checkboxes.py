#!/usr/bin/env python3
"""Add checkboxes to all implementation steps in TODO.md"""

import re
import sys

def add_checkboxes_to_line(line):
    """Add checkboxes to lines that need them"""
    
    # Pattern 1: Numbered steps like "1. **Step Name** (time):"
    if re.match(r'^(\d+)\.\s+\*\*', line):
        if not line.strip().startswith('- [ ]'):
            # Extract the number and the rest
            match = re.match(r'^(\d+)\.\s+(.*)$', line)
            if match:
                return f"{match.group(1)}. [ ] {match.group(2)}\n"
    
    # Pattern 2: Sub-items with 3 spaces + dash (main sub-items)
    elif re.match(r'^   -\s+', line) and not re.match(r'^   -\s+\[.\]', line):
        # Replace "   - " with "   - [ ] "
        return re.sub(r'^   -\s+', '   - [ ] ', line)
    
    # Pattern 3: Sub-items with 5 spaces + dash (nested sub-items)
    elif re.match(r'^     -\s+', line) and not re.match(r'^     -\s+\[.\]', line):
        # Replace "     - " with "     - [ ] "
        return re.sub(r'^     -\s+', '     - [ ] ', line)
    
    # Pattern 4: Sub-items with 2 spaces + dash (alternate format)
    elif re.match(r'^  -\s+', line) and not re.match(r'^  -\s+\[.\]', line):
        # Replace "  - " with "  - [ ] "
        return re.sub(r'^  -\s+', '  - [ ] ', line)
    
    # Pattern 5: Acceptance criteria and similar sections
    elif re.match(r'^-\s+\[.\]\s+', line):
        # Already has checkbox, keep as is
        return line
    elif re.match(r'^-\s+[A-Z]', line) and not re.match(r'^-\s+\[.\]', line):
        # Lines starting with "- " followed by capital letter (likely criteria)
        return re.sub(r'^-\s+', '- [ ] ', line)
    
    return line

def process_file(input_file, output_file):
    """Process the TODO.md file and add checkboxes"""
    
    with open(input_file, 'r') as f:
        lines = f.readlines()
    
    processed_lines = []
    changes_made = 0
    
    for i, line in enumerate(lines):
        original_line = line
        processed_line = add_checkboxes_to_line(line)
        
        if original_line != processed_line:
            changes_made += 1
            print(f"Line {i+1}: {original_line.strip()} -> {processed_line.strip()}")
        
        processed_lines.append(processed_line)
    
    with open(output_file, 'w') as f:
        f.writelines(processed_lines)
    
    print(f"\nTotal changes made: {changes_made}")
    return changes_made

if __name__ == "__main__":
    input_file = "/Users/spuri/projects/lexlapax/rs-llmspell/TODO.md"
    output_file = "/Users/spuri/projects/lexlapax/rs-llmspell/TODO.md"
    
    print("Adding checkboxes to TODO.md implementation steps...")
    changes = process_file(input_file, output_file)
    
    if changes > 0:
        print(f"Successfully added {changes} checkboxes to TODO.md")
    else:
        print("No changes needed - all checkboxes already present")