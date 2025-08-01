#!/usr/bin/env python3
"""Add security and performance categories to appropriate tests."""

import re
from pathlib import Path
from typing import List

def is_security_test(file_path: Path) -> bool:
    """Check if a test file is security-related."""
    file_name = file_path.name.lower()
    file_content = file_path.read_text().lower()
    
    security_indicators = [
        # File name patterns
        'security' in file_name,
        'auth' in file_name and 'test' in file_name,
        'injection' in file_name,
        'sandbox' in file_name,
        'validation' in file_name and ('security' in file_content or 'auth' in content),
        
        # Content patterns
        'sandbox' in file_content and ('escape' in file_content or 'security' in file_content),
        'injection' in file_content and ('attack' in file_content or 'security' in file_content),
        'authentication' in file_content,
        'authorization' in file_content,
        'security' in file_content and ('test' in file_content or 'validate' in file_content),
        'path_traversal' in file_content,
        'xss' in file_content,
        'csrf' in file_content,
        'sql_injection' in file_content,
        'command_injection' in file_content,
    ]
    
    return any(security_indicators)

def is_performance_test(file_path: Path) -> bool:
    """Check if a test file is performance-related."""
    file_name = file_path.name.lower()
    file_content = file_path.read_text().lower()
    
    performance_indicators = [
        # File name patterns
        'performance' in file_name,
        'timeout' in file_name,
        'stress' in file_name,
        'load' in file_name,
        'latency' in file_name,
        
        # Content patterns (but exclude benchmarks)
        ('performance' in file_content and 'test' in file_content and 'criterion' not in file_content),
        'timeout' in file_content and ('test' in file_content or 'assert' in file_content),
        'stress' in file_content and 'test' in file_content,
        'concurrent' in file_content and ('performance' in file_content or 'load' in file_content),
        'rate_limit' in file_content and 'test' in file_content,
        'circuit_breaker' in file_content and 'test' in file_content,
        'latency' in file_content and 'test' in file_content,
        'throughput' in file_content and 'test' in file_content and 'criterion' not in file_content,
    ]
    
    return any(performance_indicators)

def add_specialty_category(file_path: Path, category: str) -> bool:
    """Add security or performance category to tests in a file."""
    content = file_path.read_text()
    
    # Don't add if already has this category
    if f'test_category = "{category}"' in content:
        return False
    
    # Pattern to match test functions with existing categories
    # Look for functions that already have test categories
    pattern = r'((?:#\[cfg_attr\(test_category = "[^"]+"\)\]\s*\n)+)(#\[(?:tokio::)?test\])'
    
    # Add the specialty category after existing categories
    def replacement(match):
        existing_attrs = match.group(1)
        test_attr = match.group(2)
        return f'{existing_attrs}#[cfg_attr(test_category = "{category}")]\n{test_attr}'
    
    new_content = re.sub(pattern, replacement, content)
    
    if new_content != content:
        file_path.write_text(new_content)
        return True
    return False

def main():
    """Add security and performance categories to appropriate tests."""
    root = Path("/Users/spuri/projects/lexlapax/rs-llmspell")
    
    # Find all test files
    test_files = []
    for pattern in ["**/tests/**/*.rs", "**/src/**/*.rs"]:
        for file_path in root.glob(pattern):
            # Skip benchmark files
            if 'benches/' in str(file_path):
                continue
            
            content = file_path.read_text()
            # Only process files that have test categories already
            if 'test_category =' in content and ('test' in content.lower() or 'tokio::test' in content):
                test_files.append(file_path)
    
    print(f"Found {len(test_files)} categorized test files to analyze")
    
    security_files = []
    performance_files = []
    
    # Analyze each file
    for file_path in test_files:
        try:
            if is_security_test(file_path):
                security_files.append(file_path)
            if is_performance_test(file_path):
                performance_files.append(file_path)
        except Exception as e:
            print(f"Error analyzing {file_path}: {e}")
    
    print(f"\nFound {len(security_files)} security test files")
    print(f"Found {len(performance_files)} performance test files")
    
    # Add security categories
    security_updated = 0
    for file_path in security_files:
        if add_specialty_category(file_path, "security"):
            print(f"Added security category: {file_path.relative_to(root)}")
            security_updated += 1
    
    # Add performance categories  
    performance_updated = 0
    for file_path in performance_files:
        if add_specialty_category(file_path, "performance"):
            print(f"Added performance category: {file_path.relative_to(root)}")
            performance_updated += 1
    
    print(f"\nSummary:")
    print(f"Security categories added: {security_updated}")
    print(f"Performance categories added: {performance_updated}")
    print(f"Total specialty categories added: {security_updated + performance_updated}")

if __name__ == "__main__":
    main()