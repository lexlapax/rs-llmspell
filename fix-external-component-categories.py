#!/usr/bin/env python3
"""Fix component categories for external tests with #[ignore]."""

import re
from pathlib import Path

def get_crate_component(crate_name: str) -> str:
    """Map crate name to component category."""
    mapping = {
        'llmspell-tools': 'tool',
        'llmspell-agents': 'agent',
        'llmspell-workflows': 'workflow',
        'llmspell-bridge': 'bridge',
        'llmspell-hooks': 'hook',
        'llmspell-events': 'event',
        'llmspell-sessions': 'session',
        'llmspell-state-persistence': 'state',
        'llmspell-utils': 'util',
        'llmspell-core': 'core',
        'llmspell-testing': 'testing'
    }
    return mapping.get(crate_name, '')

def add_component_to_external_tests(file_path: Path, component: str) -> bool:
    """Add component category to external tests that have #[ignore]."""
    content = file_path.read_text()
    
    # Don't add if already has this component category
    if f'test_category = "{component}"' in content:
        return False
    
    modified = False
    
    # Pattern for external tests with #[ignore]
    # #[cfg_attr(test_category = "external")]
    # #[ignore]
    # #[test] or #[tokio::test]
    pattern = r'(#\[cfg_attr\(test_category = "external"\)\]\s*\n)(#\[ignore\]\s*\n)(#\[(?:tokio::)?test\])'
    replacement = r'\1#[cfg_attr(test_category = "' + component + r'")]\n\2\3'
    
    new_content = re.sub(pattern, replacement, content)
    if new_content != content:
        modified = True
        content = new_content
    
    # Also handle external tests without #[ignore]
    pattern2 = r'(#\[cfg_attr\(test_category = "external"\)\]\s*\n)(?!#\[cfg_attr)(#\[(?:tokio::)?test\])'
    replacement2 = r'\1#[cfg_attr(test_category = "' + component + r'")]\n\2'
    
    new_content = re.sub(pattern2, replacement2, content)
    if new_content != content:
        modified = True
    
    if modified:
        file_path.write_text(new_content)
    
    return modified

def process_crate_external_tests(root: Path, crate_name: str) -> int:
    """Process external tests in a crate."""
    component = get_crate_component(crate_name)
    if not component:
        return 0
    
    crate_path = root / crate_name
    if not crate_path.exists():
        return 0
    
    updated_count = 0
    
    # Process tests in tests/ directory
    tests_dir = crate_path / "tests"
    if tests_dir.exists():
        for test_file in tests_dir.rglob("*.rs"):
            # Check if file has external tests
            content = test_file.read_text()
            if 'test_category = "external"' in content:
                if add_component_to_external_tests(test_file, component):
                    print(f"  Fixed: {test_file.relative_to(root)}")
                    updated_count += 1
    
    return updated_count

def main():
    """Fix component categories for external tests."""
    root = Path("/Users/spuri/projects/lexlapax/rs-llmspell")
    
    crates = [
        'llmspell-tools',
        'llmspell-agents', 
        'llmspell-workflows',
        'llmspell-bridge',
        'llmspell-hooks',
        'llmspell-events',
        'llmspell-sessions',
        'llmspell-state-persistence',
        'llmspell-utils',
        'llmspell-core',
        'llmspell-testing'
    ]
    
    total_updated = 0
    
    for crate_name in crates:
        count = process_crate_external_tests(root, crate_name)
        if count > 0:
            print(f"\n{crate_name}: {count} external tests fixed")
        total_updated += count
    
    print(f"\nTotal external tests fixed: {total_updated}")

if __name__ == "__main__":
    main()