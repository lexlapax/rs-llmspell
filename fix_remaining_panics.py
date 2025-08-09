files_and_lines = [
    ("composition/capabilities.rs", 473),
    ("composition/hierarchical.rs", 119),
    ("composition/hierarchical.rs", 569),
    ("composition/tool_composition.rs", 316),
    ("di.rs", 193),
    ("lifecycle/benchmarks.rs", 28),
    ("lifecycle/benchmarks.rs", 80),
    ("lifecycle/shutdown.rs", 213),
    ("monitoring/events.rs", 415),
    ("monitoring/performance.rs", 98),
    ("monitoring/performance.rs", 395),
    ("registry/discovery.rs", 244),
    ("templates/mod.rs", 51),
    ("testing/mocks.rs", 621),
]

for filepath, line_num in files_and_lines:
    print(f"=== {filepath}:{line_num} ===")
    full_path = f"/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-agents/src/{filepath}"
    with open(full_path) as f:
        lines = f.readlines()
        # Show context around the line
        start = max(0, line_num - 5)
        end = min(len(lines), line_num + 2)
        for i in range(start, end):
            print(f"{i+1:4}: {lines[i]}", end="")
    print()
