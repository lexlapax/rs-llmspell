import re
import os

input_path = '/Users/spuri/projects/lexlapax/rs-llmspell/docs/rs-aikit-docs/chatgpt-agent-spec-recommendation.md'
output_path = '/Users/spuri/projects/lexlapax/rs-llmspell/docs/rs-aikit-docs/chatgpt-agent-spec-recommendation-v2.md'

with open(input_path, 'r') as f:
    content = f.read()

# Dictionary to store references: id -> url
refs = {}

def replacer(match):
    ref_id = match.group(1)
    url = match.group(2)
    # Check if we already have this ID with a different URL (unlikely but possible)
    if ref_id in refs and refs[ref_id] != url:
        print(f"Warning: Reference ID {ref_id} has multiple URLs. Overwriting.")
        print(f"Old: {refs[ref_id]}")
        print(f"New: {url}")
    
    refs[ref_id] = url
    return f"[{ref_id}]"

# Regex to match [\[N\]](url)
# Note: In the file, [ is literal, then \[ which is literal, then digits, then \] literal, then ] literal.
# Python regex string: needs escaping.
# pattern = r"\[\\\[(\d+)\\\]\]\((.*?)\)"
pattern = r"\[\\\[(\d+)\\\]\]\(([^)]+)\)"

new_content = re.sub(pattern, replacer, content)

# Append references at the end
# Check if file ends with newline
if not new_content.endswith('\n'):
    new_content += '\n'

new_content += "\n## Referenced Links\n\n"

# Sort references by ID numerically
sorted_ids = sorted(refs.keys(), key=lambda x: int(x))

for ref_id in sorted_ids:
    url = refs[ref_id]
    new_content += f"[{ref_id}]: {url}\n"

with open(output_path, 'w') as f:
    f.write(new_content)

print(f"Processed {len(refs)} references.")
print(f"Written to {output_path}")
