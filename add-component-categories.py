#!/usr/bin/env python3
"""Add component-specific categories to tests."""

import re
from pathlib import Path
from typing import List, Tuple

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

def add_component_category(file_path: Path, component: str) -> bool:
    """Add component category to tests in a file."""
    content = file_path.read_text()
    
    # Don't add if already has this component category
    if f'test_category = "{component}"' in content:
        return False
    
    # Pattern to match test functions with existing categories
    # Match patterns like:
    # #[cfg_attr(test_category = "unit")]
    # #[test] or #[tokio::test]
    pattern = r'(#\[cfg_attr\(test_category = "(?:unit|integration|external)"\)\]\s*\n)(#\[(?:tokio::)?test\])'
    
    # Add component category after the existing category
    replacement = r'\1#[cfg_attr(test_category = "' + component + r'")]\n\2'
    
    new_content = re.sub(pattern, replacement, content)
    
    # Also handle module-level unit tests in src files
    if '/src/' in str(file_path) and '#[cfg(test)]' in content:
        # Add component category to the module
        mod_pattern = r'(#\[cfg\(test\)\]\s*\n)'
        mod_replacement = r'\1#[cfg_attr(test_category = "' + component + r'")]\n'
        new_content = re.sub(mod_pattern, mod_replacement, new_content)
    
    if new_content != content:
        file_path.write_text(new_content)
        return True
    return False

def process_crate_tests(root: Path, crate_name: str) -> Tuple[int, List[str]]:
    """Process all test files in a crate."""
    component = get_crate_component(crate_name)
    if not component:
        return 0, []
    
    crate_path = root / crate_name
    if not crate_path.exists():
        return 0, []
    
    updated_files = []
    
    # Process tests in tests/ directory
    tests_dir = crate_path / "tests"
    if tests_dir.exists():
        for test_file in tests_dir.rglob("*.rs"):
            if test_file.name == "mod.rs":
                continue
            if add_component_category(test_file, component):
                updated_files.append(str(test_file.relative_to(root)))
    
    # Process unit tests in src/ directory
    src_dir = crate_path / "src"
    if src_dir.exists():
        for src_file in src_dir.rglob("*.rs"):
            # Check if file contains tests
            content = src_file.read_text()
            if '#[test]' in content or '#[cfg(test)]' in content:
                if add_component_category(src_file, component):
                    updated_files.append(str(src_file.relative_to(root)))
    
    return len(updated_files), updated_files

def main():
    """Add component categories to all tests."""
    root = Path("/Users/spuri/projects/lexlapax/rs-llmspell")
    
    # Process each crate
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
        count, files = process_crate_tests(root, crate_name)
        if count > 0:
            print(f"\n{crate_name}: {count} files updated")
            for file in files[:5]:  # Show first 5
                print(f"  - {file}")
            if count > 5:
                print(f"  ... and {count - 5} more")
        total_updated += count
    
    print(f"\nTotal files updated: {total_updated}")

if __name__ == "__main__":
    main()