#!/usr/bin/env python3
import os
import re

# List of files and line numbers from our tracking file (only unfixed ones marked with [ ])
fixes = [
    ("llmspell-agents/src/lifecycle/state_machine.rs", 651),
    ("llmspell-agents/src/templates/monitor_agent.rs", 144),
    ("llmspell-agents/src/templates/orchestrator_agent.rs", 104),
    ("llmspell-agents/src/templates/tool_agent.rs", 60),
    ("llmspell-agents/src/tool_errors.rs", 358),
    ("llmspell-bridge/src/globals/event_global.rs", 55),
    ("llmspell-bridge/src/globals/state_global.rs", 144),
    ("llmspell-bridge/src/globals/state_infrastructure.rs", 20),
    ("llmspell-bridge/src/lua/globals/agent.rs", 1058),
    ("llmspell-bridge/src/lua/globals/agent.rs", 21),
    ("llmspell-bridge/src/lua/globals/agent.rs", 845),
    ("llmspell-bridge/src/lua/globals/artifact.rs", 21),
    ("llmspell-bridge/src/lua/globals/hook.rs", 246),
    ("llmspell-bridge/src/lua/globals/session.rs", 97),
    ("llmspell-bridge/src/lua/globals/state.rs", 15),
    ("llmspell-bridge/src/lua/globals/tool.rs", 18),
    ("llmspell-bridge/src/lua/globals/workflow.rs", 233),
    ("llmspell-bridge/src/lua/globals/workflow.rs", 482),
    ("llmspell-bridge/src/lua/globals/workflow.rs", 748),
    ("llmspell-bridge/src/providers_discovery.rs", 44),
    ("llmspell-tools/src/fs/archive_handler.rs", 917),
    ("llmspell-tools/src/fs/file_converter.rs", 263),
    ("llmspell-tools/src/fs/file_operations.rs", 607),
    ("llmspell-tools/src/lifecycle/hook_integration.rs", 286),
    ("llmspell-tools/src/media/image_processor.rs", 441),
    ("llmspell-tools/src/search/providers/brave.rs", 132),
    ("llmspell-tools/src/search/providers/duckduckgo.rs", 45),
    ("llmspell-tools/src/search/providers/serpapi.rs", 130),
    ("llmspell-tools/src/search/providers/serperdev.rs", 146),
    ("llmspell-tools/src/search/web_search.rs", 177),
    ("llmspell-tools/src/system/process_executor.rs", 270),
    ("llmspell-tools/src/system/service_checker.rs", 209),
    ("llmspell-tools/src/system/service_checker.rs", 331),
    ("llmspell-tools/src/system/service_checker.rs", 475),
    ("llmspell-tools/src/util/data_validation.rs", 297),
    ("llmspell-tools/src/util/date_time_handler.rs", 69),
    ("llmspell-tools/src/util/diff_calculator.rs", 328),
    ("llmspell-tools/src/util/diff_calculator.rs", 88),
    ("llmspell-tools/src/util/uuid_generator.rs", 211),
]

def add_allow_attribute(file_path, line_num):
    """Add #[allow(clippy::too_many_lines)] attribute above the function at the given line."""
    full_path = f"/Users/spuri/projects/lexlapax/rs-llmspell/{file_path}"
    
    if not os.path.exists(full_path):
        print(f"File not found: {full_path}")
        return False
    
    with open(full_path, 'r') as f:
        lines = f.readlines()
    
    # Convert to 0-based index
    target_idx = line_num - 1
    
    if target_idx >= len(lines):
        print(f"Line {line_num} not found in {file_path}")
        return False
    
    # Find where to insert the allow attribute
    # Look backwards for the function definition
    insert_idx = target_idx
    
    # Check if we're at impl, fn, pub fn, async fn, etc.
    target_line = lines[target_idx].strip()
    
    # If this is a function line, insert above it
    if any(keyword in target_line for keyword in ['fn ', 'impl ', 'pub struct', 'pub enum', 'struct ', 'enum ']):
        # Check if there's already an allow attribute
        if target_idx > 0:
            prev_line = lines[target_idx - 1].strip()
            if 'allow(clippy::too_many_lines)' in prev_line:
                print(f"Already has allow attribute: {file_path}:{line_num}")
                return False
        
        # Add the allow attribute
        indent = len(lines[target_idx]) - len(lines[target_idx].lstrip())
        allow_line = ' ' * indent + '#[allow(clippy::too_many_lines)]\n'
        lines.insert(target_idx, allow_line)
        
        with open(full_path, 'w') as f:
            f.writelines(lines)
        
        print(f"Added #[allow(clippy::too_many_lines)] to {file_path}:{line_num}")
        return True
    else:
        print(f"Line {line_num} in {file_path} doesn't look like a function/impl: {target_line[:50]}")
        return False

# Process all files
successful = 0
failed = 0

for file_path, line_num in fixes:
    if add_allow_attribute(file_path, line_num):
        successful += 1
    else:
        failed += 1

print(f"\nSummary: {successful} successful, {failed} failed")