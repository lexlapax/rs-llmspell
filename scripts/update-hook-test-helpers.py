#!/usr/bin/env python3
"""Update hook test helpers to use centralized llmspell-testing utilities."""

import os
import re
import sys

def update_hook_test_helpers(file_path):
    """Update a single file to use centralized test helpers."""
    with open(file_path, 'r') as f:
        content = f.read()
    
    original_content = content
    
    # Pattern to find create_test_context function definitions
    pattern = r'(\s*)fn create_test_context\(\) -> HookContext \{[^}]+\}'
    
    # Check if this file has a create_test_context function
    if not re.search(pattern, content):
        return False
    
    # Add import if not already present
    if 'llmspell_testing::hook_helpers' not in content:
        # Find the test module declaration
        test_module_pattern = r'#\[cfg\(test\)\]\s*mod tests \{'
        match = re.search(test_module_pattern, content)
        if match:
            # Add import after use super::*;
            import_pattern = r'(use super::\*;)'
            if re.search(import_pattern, content):
                content = re.sub(
                    import_pattern,
                    r'\1\n    use llmspell_testing::hook_helpers::create_test_hook_context;',
                    content
                )
            else:
                # Add after the mod tests {
                content = re.sub(
                    test_module_pattern,
                    r'#[cfg(test)]\nmod tests {\n    use super::*;\n    use llmspell_testing::hook_helpers::create_test_hook_context;',
                    content
                )
    
    # Replace the create_test_context function with a simpler wrapper
    content = re.sub(
        pattern,
        r'\1fn create_test_context() -> HookContext {\n\1    create_test_hook_context()\n\1}',
        content,
        flags=re.DOTALL
    )
    
    if content != original_content:
        with open(file_path, 'w') as f:
            f.write(content)
        return True
    
    return False

def main():
    hook_files = [
        '/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-hooks/src/builtin/caching.rs',
        '/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-hooks/src/builtin/cost_tracking.rs',
        '/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-hooks/src/builtin/retry.rs',
        '/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-hooks/src/cache/mod.rs',
        '/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-hooks/src/persistence/tests.rs',
    ]
    
    updated = 0
    for file_path in hook_files:
        if os.path.exists(file_path):
            if update_hook_test_helpers(file_path):
                print(f"Updated: {file_path}")
                updated += 1
            else:
                print(f"No changes needed: {file_path}")
        else:
            print(f"File not found: {file_path}")
    
    print(f"\nTotal files updated: {updated}")
    return 0

if __name__ == '__main__':
    sys.exit(main())