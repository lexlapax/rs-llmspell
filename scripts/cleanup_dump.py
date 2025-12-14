import os

file_path = '/Users/spuri/projects/lexlapax/rs-llmspell/docs/rs-aikit-docs/chatgpt-agent-spec-recommendation-v2.md'

with open(file_path, 'r') as f:
    lines = f.readlines()

# Find the start index (line with '---' after line 315)
start_idx = -1
for i, line in enumerate(lines):
    if i > 315 and line.strip() == '---':
        start_idx = i
        break

# Find the end index (line with '## Referenced Links')
end_idx = -1
for i, line in enumerate(lines):
    if line.strip() == '## Referenced Links':
        end_idx = i
        break

if start_idx != -1 and end_idx != -1:
    # Keep the '---' and '## Referenced Links'
    # Remove lines between them
    new_lines = lines[:start_idx+1] + ['\n'] + lines[end_idx:]
    
    with open(file_path, 'w') as f:
        f.writelines(new_lines)
    print("Removed detailed dump section.")
else:
    print("Could not find section boundaries.")
    print(f"Start: {start_idx}, End: {end_idx}")
