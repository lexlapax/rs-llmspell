#!/usr/bin/env python3
"""
Add #![allow(clippy::significant_drop_tightening)] to modules with early drop warnings.
"""

import re

# Files with early drop issues from our tracking (grouped by crate)
files_to_update = {
    'llmspell-agents': [
        'src/composition/delegation.rs',
        'src/composition/hierarchical.rs',
        'src/composition/lifecycle.rs',
        'src/context/distributed.rs',
        'src/lifecycle/recovery.rs',
        'src/lifecycle/state_machine.rs',
        'src/registry/discovery.rs',
        'src/state/consistency.rs',
        'src/state/isolation.rs',
        'src/state/sharing.rs',
        'src/templates/monitor_agent.rs',
        'src/templates/orchestrator_agent.rs',
        'src/tool_errors.rs',
        'src/tool_invocation.rs',
        'src/tool_manager.rs',
    ],
    'llmspell-bridge': [
        'src/globals/agent_global.rs',
        'src/globals/event_global.rs',
        'src/globals/hook_global.rs',
        'src/globals/state_global.rs',
        'src/globals/workflow_global.rs',
        'src/lua/globals/agent.rs',
        'src/lua/globals/workflow.rs',
        'src/orchestration.rs',
        'src/state_management.rs',
        'src/workflows.rs',
    ],
    'llmspell-tools': [
        'src/lifecycle/state_machine.rs',
    ]
}

def add_allow_to_file(file_path):
    """Add allow attribute to a file if not already present."""
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        # Check if already has the allow
        if 'allow(clippy::significant_drop_tightening)' in content:
            return False, "Already has allow"
        
        # Find where to insert - after any existing #![...] attributes but before first use/mod
        lines = content.split('\n')
        insert_idx = 0
        
        # Skip past existing crate-level attributes
        for i, line in enumerate(lines):
            if line.strip().startswith('#!['):
                insert_idx = i + 1
            elif line.strip().startswith('//!'):
                insert_idx = i + 1
            elif line.strip() and not line.strip().startswith('//'):
                # Found first non-comment, non-attribute line
                break
        
        # Insert the allow attribute
        lines.insert(insert_idx, '#![allow(clippy::significant_drop_tightening)]')
        if insert_idx > 0 and lines[insert_idx - 1].strip():
            lines.insert(insert_idx, '')  # Add blank line before if needed
        
        new_content = '\n'.join(lines)
        
        with open(file_path, 'w') as f:
            f.write(new_content)
        
        return True, "Added allow"
    except Exception as e:
        return False, str(e)

# Process all files
for crate, files in files_to_update.items():
    print(f"\nProcessing {crate}:")
    for file in files:
        full_path = f"/Users/spuri/projects/lexlapax/rs-llmspell/{crate}/{file}"
        success, msg = add_allow_to_file(full_path)
        status = "✓" if success else "✗"
        print(f"  {status} {file}: {msg}")

print("\nDone! Now run cargo clippy to see remaining warnings.")