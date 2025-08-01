#!/usr/bin/env python3
"""Fix miscategorized tests that should be marked as external."""

import re
from pathlib import Path

def fix_file(file_path, old_category, new_category):
    """Fix test category in a file."""
    content = file_path.read_text()
    
    # Pattern to match test functions with wrong category
    pattern = r'(#\[cfg_attr\(test_category = "' + old_category + r'"\)\]\s*\n)(#\[(?:tokio::)?test\])'
    
    # Replace with correct category
    replacement = r'#[cfg_attr(test_category = "' + new_category + r'")]\n\2'
    
    new_content = re.sub(pattern, replacement, content)
    
    if new_content != content:
        file_path.write_text(new_content)
        return True
    return False

def main():
    """Fix miscategorized external tests."""
    root = Path("/Users/spuri/projects/lexlapax/rs-llmspell")
    
    # Files that use httpbin.org but are marked as integration
    files_to_fix = [
        "llmspell-tools/tests/webhook_caller_integration.rs",
        "llmspell-tools/tests/webpage_monitor_integration.rs", 
        "llmspell-tools/tests/web_tools_error_scenarios.rs"
    ]
    
    fixed_count = 0
    
    for file_path in files_to_fix:
        full_path = root / file_path
        if full_path.exists():
            if fix_file(full_path, "integration", "external"):
                print(f"Fixed: {file_path}")
                fixed_count += 1
            else:
                print(f"No changes needed: {file_path}")
        else:
            print(f"File not found: {file_path}")
    
    # Fix tests in common/mod.rs that are wrongly marked as external
    common_path = root / "llmspell-tools/tests/common/mod.rs"
    if common_path.exists():
        content = common_path.read_text()
        
        # These are simple unit tests, not external
        pattern = r'#\[cfg_attr\(test_category = "external"\)\]\s*\n\s*#\[ignore\]\s*\n\s*#\[test\]'
        replacement = '#[cfg_attr(test_category = "unit")]\n    #[test]'
        
        new_content = re.sub(pattern, replacement, content)
        
        if new_content != content:
            common_path.write_text(new_content)
            print(f"Fixed: llmspell-tools/tests/common/mod.rs")
            fixed_count += 1
    
    print(f"\nTotal files fixed: {fixed_count}")

if __name__ == "__main__":
    main()