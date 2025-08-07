#!/usr/bin/env python3
"""
Script to systematically add scope blocks to fix early drop warnings.
The pattern is to wrap lock acquisitions in their own scope block.
"""

import re

# Common patterns that need early drop fixes
patterns_to_fix = [
    # Pattern 1: RwLock write that should be scoped
    (r'(\s+)(let mut \w+ = self\.\w+\.write\(\)\.unwrap\(\);[^}]+)(\n\s+Ok\(|return|Some\()',
     r'\1{\n\1    \2\n\1}\n\1\3'),
    
    # Pattern 2: RwLock read that should be scoped  
    (r'(\s+)(let \w+ = self\.\w+\.read\(\)(?:\.await)?;[^}]+)(\n\s+Ok\(|return|Some\()',
     r'\1{\n\1    \2\n\1}\n\1\3'),
]

def fix_early_drops(file_path):
    """Fix early drop issues in a file."""
    with open(file_path, 'r') as f:
        content = f.read()
    
    original = content
    fixes_made = 0
    
    # Manual fixes for specific patterns we identified
    # Fix 1: Wrap metrics updates in blocks
    if 'metrics.write().unwrap()' in content:
        # This is more complex - would need manual review
        pass
    
    # Count potential fixes
    potential_fixes = content.count('.write().unwrap()') + content.count('.read().await')
    
    return potential_fixes

# Files with early drop issues from our tracking
files_to_check = [
    'llmspell-agents/src/composition/hierarchical.rs',
    'llmspell-agents/src/composition/lifecycle.rs',
    'llmspell-agents/src/context/distributed.rs',
    'llmspell-agents/src/lifecycle/recovery.rs',
    'llmspell-agents/src/lifecycle/state_machine.rs',
    'llmspell-agents/src/registry/discovery.rs',
    'llmspell-agents/src/state/consistency.rs',
    'llmspell-agents/src/state/isolation.rs',
    'llmspell-agents/src/state/sharing.rs',
    'llmspell-agents/src/templates/monitor_agent.rs',
    'llmspell-agents/src/templates/orchestrator_agent.rs',
    'llmspell-agents/src/tool_errors.rs',
    'llmspell-agents/src/tool_invocation.rs',
    'llmspell-agents/src/tool_manager.rs',
]

print("Files with early drop issues to fix manually:")
for file in files_to_check:
    full_path = f"/Users/spuri/projects/lexlapax/rs-llmspell/{file}"
    print(f"- {file}")

print("\nGeneral fix pattern:")
print("Wrap lock acquisitions in scope blocks:")
print("BEFORE:")
print("  let mut lock = self.field.write().unwrap();")
print("  lock.do_something();")
print("  Ok(result)")
print("\nAFTER:")
print("  {")
print("      let mut lock = self.field.write().unwrap();")
print("      lock.do_something();")
print("  }")
print("  Ok(result)")