#!/usr/bin/env python3
"""
Migrate test utilities to use consolidated helpers from llmspell-testing.

This script updates test code to:
1. Remove duplicate create_test_tool() functions
2. Remove duplicate create_test_input() functions  
3. Add proper imports from llmspell-testing
4. Update test code to use the consolidated helpers
"""

import os
import re
from pathlib import Path
from typing import List, Tuple

# Patterns to find and replace
PATTERNS = [
    # Remove create_test_tool functions in tool tests
    (
        r'(\s*)fn create_test_tool\(\)[^{]*\{[^}]*\}',
        '',
        'tool_test_files'
    ),
    # Remove create_test_input functions
    (
        r'(\s*)fn create_test_input\([^)]*\)[^{]*\{[^}]*?parameters:\s*\{[^}]*\}[^}]*\}[^}]*\}',
        '',
        'tool_test_files'
    ),
    # Add llmspell-testing imports for tool tests
    (
        r'(#\[cfg\(test\)\]\s*mod tests\s*\{)',
        r'\1\n    use llmspell_testing::tool_helpers::{create_test_tool, create_test_tool_input};',
        'tool_test_files'
    ),
    # Update tool creation to use helper
    (
        r'let tool = create_test_tool\(\);',
        r'let tool = create_test_tool(\n            &self.metadata().name,\n            &self.metadata().description,\n            vec![]\n        );',
        'tool_test_files'
    ),
    # Update create_test_input calls to use helper
    (
        r'create_test_input\("([^"]*)",\s*serde_json::json!\(\{([^}]*)\}\)\)',
        r'create_test_tool_input(vec![\2])',
        'tool_test_files'
    ),
]

# File patterns to search
FILE_GROUPS = {
    'tool_test_files': [
        'llmspell-tools/src/**/*.rs',
    ],
    'agent_test_files': [
        'llmspell-agents/tests/**/*.rs',
        'llmspell-hooks/tests/**/*.rs',
    ],
    'state_test_files': [
        'llmspell-state-persistence/tests/**/*.rs',
        'llmspell-state-persistence/src/**/*test*.rs',
    ]
}

def find_files(pattern: str) -> List[Path]:
    """Find all files matching the given glob pattern."""
    base_dir = Path('/Users/spuri/projects/lexlapax/rs-llmspell')
    files = []
    for p in base_dir.glob(pattern):
        if p.is_file() and p.suffix == '.rs':
            files.append(p)
    return files

def update_file(file_path: Path, patterns: List[Tuple[str, str]]) -> bool:
    """Update a file with the given patterns. Returns True if changes were made."""
    try:
        content = file_path.read_text()
        original_content = content
        
        for pattern, replacement in patterns:
            # Use DOTALL flag to match across multiple lines
            content = re.sub(pattern, replacement, content, flags=re.DOTALL | re.MULTILINE)
        
        if content != original_content:
            file_path.write_text(content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Main migration function."""
    print("Starting test utility migration...")
    
    total_files_updated = 0
    
    for pattern, replacement, file_group in PATTERNS:
        if file_group not in FILE_GROUPS:
            continue
            
        print(f"\nApplying pattern for {file_group}...")
        patterns_to_apply = [(pattern, replacement)]
        
        for file_pattern in FILE_GROUPS[file_group]:
            files = find_files(file_pattern)
            print(f"  Found {len(files)} files matching {file_pattern}")
            
            for file_path in files:
                if update_file(file_path, patterns_to_apply):
                    print(f"    ✓ Updated: {file_path.relative_to(Path('/Users/spuri/projects/lexlapax/rs-llmspell'))}")
                    total_files_updated += 1
    
    print(f"\nMigration complete! Updated {total_files_updated} files.")
    
    # Add llmspell-testing as a dev dependency to crates that need it
    print("\nDon't forget to add llmspell-testing as a dev dependency:")
    print("  [dev-dependencies]")
    print("  llmspell-testing = { path = \"../llmspell-testing\" }")

if __name__ == "__main__":
    main()