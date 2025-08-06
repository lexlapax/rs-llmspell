import re

with open('all_clippy_output.txt', 'r') as f:
    lines = f.readlines()

current_file = None
warnings = []

for i, line in enumerate(lines):
    # Match file path
    if '-->' in line:
        match = re.search(r'--> (.+\.rs:\d+:\d+)', line)
        if match:
            current_file = match.group(1)
    
    # Match cast warning
    if 'warning:' in line and 'cast' in line:
        if current_file:
            # Extract warning type
            if 'cast_precision_loss' in lines[i+3] if i+3 < len(lines) else '':
                warning_type = 'cast_precision_loss'
            elif 'cast_possible_truncation' in lines[i+3] if i+3 < len(lines) else '':
                warning_type = 'cast_possible_truncation'
            elif 'cast_sign_loss' in lines[i+3] if i+3 < len(lines) else '':
                warning_type = 'cast_sign_loss'
            elif 'cast_lossless' in lines[i+3] if i+3 < len(lines) else '':
                warning_type = 'cast_lossless'
            elif 'cast_possible_wrap' in lines[i+3] if i+3 < len(lines) else '':
                warning_type = 'cast_possible_wrap'
            else:
                # Try to extract from the warning message
                if 'precision' in line:
                    warning_type = 'cast_precision_loss'
                elif 'truncat' in line:
                    warning_type = 'cast_possible_truncation'
                elif 'sign' in line:
                    warning_type = 'cast_sign_loss'
                elif 'infallibly' in line:
                    warning_type = 'cast_lossless'
                elif 'wrap' in line:
                    warning_type = 'cast_possible_wrap'
                else:
                    warning_type = 'unknown_cast'
            
            warnings.append(f"{current_file} {warning_type}")

# Remove duplicates and sort
warnings = sorted(set(warnings))

with open('phase_10_3_tracking.txt', 'w') as f:
    for w in warnings:
        f.write(w + '\n')

print(f"Found {len(warnings)} unique cast warnings")
