#!/usr/bin/env python3
"""Remove redundant #[ignore] from categorized external tests."""

import re
from pathlib import Path

def remove_redundant_ignore(file_path: Path) -> int:
    """Remove redundant #[ignore] from external tests in a file."""
    content = file_path.read_text()
    
    # Pattern to match external tests with redundant #[ignore]
    # Look for:
    # #[cfg_attr(test_category = "external")]
    # possibly more #[cfg_attr(...)] lines
    # #[ignore]  <- this one we want to remove (but keep #[ignore = "reason"])
    # #[test] or #[tokio::test]
    
    # Count removals
    removals = 0
    
    # Split into lines for easier processing
    lines = content.split('\n')
    new_lines = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Check if this line starts an external test
        if '#[cfg_attr(test_category = "external")]' in line:
            # Found external test, collect all attributes until we hit the test function
            test_block = [line]
            i += 1
            
            # Collect all #[...] attributes
            while i < len(lines) and lines[i].strip().startswith('#['):
                attr_line = lines[i]
                
                # Check if this is a bare #[ignore] (redundant)
                if attr_line.strip() == '#[ignore]':
                    # Skip this line (remove it)
                    removals += 1
                    print(f"  Removed redundant #[ignore] from {file_path.name}")
                else:
                    # Keep this attribute (could be #[ignore = "reason"] or other attributes)
                    test_block.append(attr_line)
                i += 1
            
            # Add the function line (should be #[test] or #[tokio::test])
            if i < len(lines):
                test_block.append(lines[i])
            
            # Add all collected lines
            new_lines.extend(test_block)
        else:
            # Regular line, keep as-is
            new_lines.append(line)
        
        i += 1
    
    # Write back if changes were made
    if removals > 0:
        file_path.write_text('\n'.join(new_lines))
    
    return removals

def main():
    """Remove redundant #[ignore] from all external tests."""
    root = Path("/Users/spuri/projects/lexlapax/rs-llmspell")
    
    # Find all test files that have external tests
    test_files = []
    for pattern in ["**/tests/**/*.rs", "**/src/**/*.rs"]:
        for file_path in root.glob(pattern):
            content = file_path.read_text()
            if 'test_category = "external"' in content:
                test_files.append(file_path)
    
    print(f"Found {len(test_files)} files with external tests")
    
    total_removals = 0
    
    for test_file in test_files:
        removals = remove_redundant_ignore(test_file)
        total_removals += removals
    
    print(f"\nTotal redundant #[ignore] attributes removed: {total_removals}")
    
    # Show examples of what we kept (ignore with reasons)
    print("\nExamples of #[ignore] attributes we kept (with reasons):")
    for test_file in test_files[:5]:
        content = test_file.read_text()
        for line in content.split('\n'):
            if '#[ignore =' in line and 'external' not in line:
                print(f"  {test_file.name}: {line.strip()}")
                break

if __name__ == "__main__":
    main()