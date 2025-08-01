#!/usr/bin/env python3
"""
Script to remove invalid #[cfg_attr(test_category = "...")] lines from Rust files.

These attributes were added as part of Task 7.1.6 but use incorrect syntax.
The cfg_attr macro expects a boolean condition and attributes to apply,
but test_category = "value" is not a valid cfg condition.

This script removes all such lines to restore compilation.
"""

import sys
import re
import os
from pathlib import Path

def remove_invalid_cfg_attr(file_path):
    """Remove invalid cfg_attr lines from a Rust file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Pattern to match #[cfg_attr(test_category = "...")] including duplicates
        # This matches both simple and duplicate patterns like:
        # #[cfg_attr(test_category = "integration")]
        # #[cfg_attr(test_category = "integration", test_category = "integration")]
        pattern = r'^\s*#\[cfg_attr\(.*test_category\s*=.*\)\]\s*\n'
        
        # Remove all matching lines
        content = re.sub(pattern, '', content, flags=re.MULTILINE)
        
        # Only write if there were changes
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            
            # Count removed lines
            removed_lines = original_content.count('\n') - content.count('\n')
            print(f"✓ {file_path}: Removed {removed_lines} invalid cfg_attr line(s)")
            return True
        else:
            print(f"- {file_path}: No invalid cfg_attr lines found")
            return False
            
    except Exception as e:
        print(f"✗ Error processing {file_path}: {e}")
        return False

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 remove-invalid-cfg-attr.py <rust-file>")
        print("       find . -name '*.rs' -exec python3 remove-invalid-cfg-attr.py {} \\;")
        sys.exit(1)
    
    file_path = sys.argv[1]
    
    # Verify it's a Rust file
    if not file_path.endswith('.rs'):
        print(f"Skipping non-Rust file: {file_path}")
        sys.exit(0)
    
    # Verify file exists
    if not os.path.exists(file_path):
        print(f"File not found: {file_path}")
        sys.exit(1)
    
    remove_invalid_cfg_attr(file_path)

if __name__ == "__main__":
    main()