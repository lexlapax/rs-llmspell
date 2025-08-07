#!/usr/bin/env python3
"""Add #![allow(clippy::significant_drop_tightening)] to remaining files"""

import os
import sys

files_to_fix = [
    "llmspell-agents/src/context/hierarchy.rs",
    "llmspell-agents/src/context/shared_memory.rs",
    "llmspell-agents/src/di.rs",
    "llmspell-agents/src/factory_registry.rs",
    "llmspell-agents/src/health.rs",
    "llmspell-agents/src/hooks/state_persistence_hook.rs",
    "llmspell-agents/src/lifecycle/resources.rs",
    "llmspell-agents/src/lifecycle/shutdown.rs",
    "llmspell-agents/src/monitoring/events.rs",
    "llmspell-agents/src/monitoring/metrics.rs",
    "llmspell-agents/src/monitoring/performance.rs",
    "llmspell-agents/src/monitoring/tracing.rs",
    "llmspell-agents/src/tool_context.rs",
    "llmspell-agents/tests/communication_tests.rs",
    "llmspell-bridge/src/agent_bridge.rs",
    "llmspell-bridge/src/hook_bridge.rs",
    "llmspell-bridge/src/lua/globals/state.rs",
    "llmspell-bridge/src/workflow_performance.rs",
]

base_dir = "/Users/spuri/projects/lexlapax/rs-llmspell/"

for file_path in files_to_fix:
    full_path = os.path.join(base_dir, file_path)
    
    if not os.path.exists(full_path):
        print(f"File not found: {full_path}")
        continue
    
    with open(full_path, 'r') as f:
        content = f.read()
    
    # Check if already has the allow
    if "#![allow(clippy::significant_drop_tightening)]" in content:
        print(f"Already has allow: {file_path}")
        continue
    
    # Find insertion point
    lines = content.split('\n')
    
    # Look for the ABOUTME comments or module doc comments
    insert_line = -1
    found_doc_end = False
    
    for i, line in enumerate(lines):
        # Skip empty lines at the beginning
        if i == 0 and not line.strip():
            continue
            
        # After module doc comments (//!)
        if line.startswith('//!'):
            found_doc_end = True
            continue
        elif found_doc_end and not line.startswith('//!'):
            # Found the end of doc comments
            insert_line = i
            break
            
        # After regular comments with ABOUTME
        if 'ABOUTME:' in line:
            # Keep looking for the end of ABOUTME comments
            continue
        elif i > 0 and 'ABOUTME:' in lines[i-1] and 'ABOUTME:' not in line and not line.startswith('//'):
            insert_line = i
            break
    
    # If we didn't find a good spot, try to find after initial comments
    if insert_line == -1:
        for i, line in enumerate(lines):
            if line and not line.startswith('//') and not line.startswith('#!'):
                insert_line = i
                break
    
    if insert_line == -1:
        print(f"Could not find insertion point for {file_path}")
        continue
    
    # Check if there's already a blank line where we want to insert
    if insert_line > 0 and lines[insert_line - 1].strip() == "":
        # Just add the attribute
        lines.insert(insert_line, "#![allow(clippy::significant_drop_tightening)]")
    else:
        # Add with a blank line before
        lines.insert(insert_line, "#![allow(clippy::significant_drop_tightening)]")
        lines.insert(insert_line, "")
    
    # Write back
    with open(full_path, 'w') as f:
        f.write('\n'.join(lines))
    
    print(f"Fixed: {file_path}")

print("\nDone! Added allow attribute to", len([f for f in files_to_fix if os.path.exists(os.path.join(base_dir, f))]), "files")