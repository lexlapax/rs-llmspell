# File Classification Template

**Category**: Productivity
**Version**: 0.1.0
**Status**: Production Ready

## Overview

The File Classification Template provides automated file organization using the scan-classify-act pattern. The template scans directories, classifies files using extension-based, content-based, or hybrid strategies, and organizes them into category-based destinations with safe dry-run preview.

**Key Features:**
- **3 Classification Strategies**: Extension (fast pattern matching), content (keyword search), hybrid (combined)
- **4 Category Presets**: Documents, media, code, downloads with predefined extensions
- **Dry-Run Mode**: Preview classifications without modifying files (default: enabled)
- **3 Action Types**: Move, copy, or report-only classification
- **Recursive Scanning**: Process entire directory trees
- **Progress Reporting**: Real-time progress for bulk operations (>10 files)
- **Multiple Output Formats**: Text, markdown, JSON classification reports
- **Safe Error Handling**: Permission errors, disk space checks, invalid paths

## Quick Start

### Safety First: Dry-Run Preview

**ALWAYS** run dry-run mode first to preview classifications:

```bash
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param category_preset=downloads \
  --param classification_strategy=extension \
  --param dry_run=true
```

Output shows what **would** happen without making changes:
```
[DRY-RUN] Would move /Users/name/Downloads/archive.zip to Downloads/Archives
[DRY-RUN] Would move /Users/name/Downloads/installer.dmg to Downloads/Installers
```

### Basic Document Classification

Classify documents in a directory:

```bash
llmspell template exec file-classification \
  --param source_path=~/Documents \
  --param category_preset=documents \
  --param action=report
```

### Organize Downloads Folder

Actually move files (after dry-run verification):

```bash
# 1. First: dry-run to preview
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param category_preset=downloads \
  --param action=move \
  --param destination_base=~/Organized \
  --param dry_run=true

# 2. Review output, then execute:
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param category_preset=downloads \
  --param action=move \
  --param destination_base=~/Organized \
  --param dry_run=false
```

### Recursive Code Classification

Classify all source code files in project tree:

```bash
llmspell template exec file-classification \
  --param source_path=~/Projects/myapp \
  --param category_preset=code \
  --param classification_strategy=extension \
  --param recursive=true \
  --param action=report \
  --param output_format=json
```

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `source_path` | String | Directory or file path to classify (must exist) |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `classification_strategy` | String | `"extension"` | Strategy: `extension`, `content`, `hybrid` |
| `category_preset` | String | `"documents"` | Preset categories: `documents`, `media`, `code`, `downloads` |
| `action` | String | `"report"` | Action: `move`, `copy`, `report` |
| `destination_base` | String | - | Base path for move/copy actions (appends category paths) |
| `dry_run` | Boolean | `true` | Preview mode - no file modifications |
| `recursive` | Boolean | `false` | Scan subdirectories recursively |
| `output_format` | String | `"text"` | Report format: `text`, `markdown`, `json` |

## Classification Strategies

### 1. Extension-Based (`extension`)

**Fast** pattern matching based on file extensions.

**How it Works:**
- Matches file extension against category extension lists
- Instant classification with 1.0 confidence
- Case-insensitive matching (`.PDF` = `.pdf`)

**Pros:**
- Extremely fast (< 1ms per file)
- 100% confidence for matched files
- No file reading required

**Cons:**
- Cannot classify files without extensions
- Cannot detect file content mismatches

**Example:**
```bash
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param classification_strategy=extension \
  --param category_preset=media
```

**Use Cases:**
- Large directories (1000+ files)
- Well-named files with correct extensions
- Performance-critical operations

### 2. Content-Based (`content`)

**Thorough** classification using keyword search in file content.

**How it Works:**
- Reads first 1KB of text files
- Searches for category keywords
- Confidence based on keyword match ratio
- Falls back to extension if content unreadable

**Pros:**
- Can classify files without extensions
- Detects content-based categorization
- Useful for misnamed files

**Cons:**
- Slower (file I/O required)
- Only works with text files
- Binary files fall back to extension

**Example:**
```bash
# Custom categories with keywords
llmspell template exec file-classification \
  --param source_path=~/Documents \
  --param classification_strategy=content
```

**Use Cases:**
- Files without extensions
- Misnamed files
- Content-based organization (invoices, reports)

### 3. Hybrid (`hybrid`)

**Balanced** approach: extension first, content fallback.

**How it Works:**
1. Try extension-based classification first (fast)
2. If no extension match, try content-based (thorough)
3. Fall back to "Uncategorized" if both fail

**Pros:**
- Fast for most files (extension match)
- Thorough for edge cases (content match)
- Best of both strategies

**Cons:**
- Slightly slower than pure extension
- Still requires readable text for content fallback

**Example:**
```bash
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param classification_strategy=hybrid \
  --param category_preset=documents
```

**Use Cases:**
- Mixed file types
- Unknown file naming conventions
- General-purpose classification

## Category Presets

### 1. Documents (`documents`)

Organize common document types.

**Categories:**
- **PDFs**: `.pdf` → `Documents/PDFs`
- **Word Documents**: `.doc`, `.docx` → `Documents/Word`
- **Spreadsheets**: `.xls`, `.xlsx`, `.csv` → `Documents/Spreadsheets`
- **Text Files**: `.txt`, `.md`, `.rtf` → `Documents/Text`
- **Presentations**: `.ppt`, `.pptx` → `Documents/Presentations`

**Example:**
```bash
llmspell template exec file-classification \
  --param source_path=~/Documents \
  --param category_preset=documents \
  --param action=move \
  --param destination_base=~/Organized
```

### 2. Media (`media`)

Organize photos, videos, and audio files.

**Categories:**
- **Photos**: `.jpg`, `.jpeg`, `.png`, `.gif`, `.bmp`, `.webp` → `Media/Photos`
- **Videos**: `.mp4`, `.avi`, `.mov`, `.mkv`, `.wmv` → `Media/Videos`
- **Audio**: `.mp3`, `.wav`, `.flac`, `.aac`, `.m4a` → `Media/Audio`

**Example:**
```bash
llmspell template exec file-classification \
  --param source_path=~/Pictures \
  --param category_preset=media \
  --param recursive=true
```

### 3. Code (`code`)

Organize source code by programming language.

**Categories:**
- **Rust**: `.rs`, `.toml` → `Code/Rust`
- **Python**: `.py`, `.pyw` → `Code/Python`
- **JavaScript**: `.js`, `.jsx`, `.ts`, `.tsx` → `Code/JavaScript`
- **Go**: `.go` → `Code/Go`
- **Other Code**: `.java`, `.c`, `.cpp`, `.h`, `.hpp` → `Code/Other`

**Example:**
```bash
llmspell template exec file-classification \
  --param source_path=~/Projects/snippets \
  --param category_preset=code \
  --param output_format=markdown
```

### 4. Downloads (`downloads`)

Organize typical download folder contents.

**Categories:**
- **Archives**: `.zip`, `.tar`, `.gz`, `.rar`, `.7z` → `Downloads/Archives`
- **Installers**: `.exe`, `.dmg`, `.pkg`, `.deb`, `.rpm` → `Downloads/Installers`
- **Documents**: `.pdf`, `.doc`, `.docx` → `Downloads/Documents`

**Example:**
```bash
# Clean up downloads folder
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param category_preset=downloads \
  --param action=move \
  --param destination_base=~/Downloads/Organized
```

## Action Types

### 1. Move (`move`)

Relocate files to category-based destinations.

**Behavior:**
- Creates destination directories automatically
- Moves files (original removed from source)
- Preserves filenames within destination

**Safety:**
- Dry-run mode prevents accidental moves
- Checks disk space before moving
- Handles permission errors gracefully

**Example:**
```bash
llmspell template exec file-classification \
  --param source_path=~/Unsorted \
  --param action=move \
  --param destination_base=~/Sorted \
  --param dry_run=false
```

### 2. Copy (`copy`)

Duplicate files to destinations while keeping originals.

**Behavior:**
- Creates destination directories
- Copies files (original remains in source)
- Useful for non-destructive organization

**Use Cases:**
- Backup while organizing
- Multi-location file organization
- Safe testing of classification

**Example:**
```bash
llmspell template exec file-classification \
  --param source_path=~/Documents \
  --param action=copy \
  --param destination_base=~/Backup/Organized
```

### 3. Report (`report`)

Classification analysis without file modifications.

**Behavior:**
- Analyzes and classifies files
- Generates detailed report
- No file system modifications
- Safe to run repeatedly

**Output Formats:**
- **Text**: Human-readable summary
- **Markdown**: Formatted report with tables
- **JSON**: Structured data for automation

**Example:**
```bash
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param action=report \
  --param output_format=json > classification_report.json
```

## Output Formats

### Text Report

Simple, human-readable summary.

```
=== FILE CLASSIFICATION REPORT ===

Total Files: 25

CATEGORY BREAKDOWN:
  PDFs → 8 files
  Photos → 12 files
  Archives → 5 files

DETAILED CLASSIFICATIONS:
  /Users/name/Downloads/file1.pdf → PDFs (confidence: 1.00)
    Destination: Documents/PDFs
```

### Markdown Report

Formatted report with tables and sections.

```markdown
# File Classification Report

**Total Files**: 25

## Category Breakdown

- **PDFs**: 8 files
- **Photos**: 12 files
- **Archives**: 5 files

## Detailed Classifications

- `/Users/name/Downloads/file1.pdf` → **PDFs** (confidence: 1.00) - Documents/PDFs
```

### JSON Report

Structured data for automation and further processing.

```json
{
  "total_files": 25,
  "categories": {
    "PDFs": 8,
    "Photos": 12,
    "Archives": 5
  },
  "classifications": [
    {
      "file_path": "/Users/name/Downloads/file1.pdf",
      "category": "PDFs",
      "confidence": 1.0,
      "destination": "Documents/PDFs",
      "action": "move"
    }
  ]
}
```

## Dry-Run Mode

### Why Dry-Run is Default

File operations are **irreversible**. Dry-run mode protects against:
- Accidental file moves
- Incorrect destination paths
- Unexpected classifications
- Disk space issues

### Workflow Best Practice

```bash
# STEP 1: Always dry-run first
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param action=move \
  --param dry_run=true

# STEP 2: Review output carefully
# Look for:
# - Correct category assignments
# - Valid destination paths
# - No surprises

# STEP 3: Execute if satisfied
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param action=move \
  --param dry_run=false
```

### Dry-Run Output

```
[DRY-RUN] Would move /path/to/file.pdf to Documents/PDFs
[DRY-RUN] Would move /path/to/photo.jpg to Media/Photos
[DRY-RUN] Would copy /path/to/code.rs to Code/Rust
```

## Progress Reporting

For large directories (>10 files), the template reports progress:

```
Phase 1: Scanning files from /Users/name/Downloads
Found 127 files to classify

Phase 2: Classifying files using extension strategy
Progress: 10/127 files classified
Progress: 20/127 files classified
Progress: 30/127 files classified
...
Classification complete: 127 files classified

Phase 3: Executing move actions (dry_run=false)
...
```

## Use Cases

### 1. Clean Downloads Folder

Organize accumulated downloads by type.

```bash
# Preview first
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param category_preset=downloads \
  --param action=move \
  --param destination_base=~/Downloads/Organized

# Execute after review
# (set dry_run=false)
```

### 2. Archive Old Project Files

Organize code files from mixed projects.

```bash
llmspell template exec file-classification \
  --param source_path=~/OldProjects \
  --param category_preset=code \
  --param recursive=true \
  --param action=copy \
  --param destination_base=~/Archive/ByLanguage
```

### 3. Photo Library Organization

Separate photos from other media.

```bash
llmspell template exec file-classification \
  --param source_path=~/Pictures/Unsorted \
  --param category_preset=media \
  --param recursive=true \
  --param action=move
```

### 4. Document Backup with Classification

Create organized backup of documents.

```bash
llmspell template exec file-classification \
  --param source_path=~/Documents \
  --param category_preset=documents \
  --param action=copy \
  --param destination_base=~/Backup/Documents \
  --param recursive=true
```

### 5. Analysis Without Modification

Generate classification report for planning.

```bash
llmspell template exec file-classification \
  --param source_path=~/Desktop \
  --param action=report \
  --param output_format=json > desktop_analysis.json
```

## Lua Integration

### Basic Classification

```lua
local Template = require("llmspell.template")

local result = Template.execute("file-classification", {
    source_path = os.getenv("HOME") .. "/Downloads",
    category_preset = "downloads",
    classification_strategy = "extension",
    action = "report",
    output_format = "json"
})

print("Total files:", result.metadata.files_scanned)
print(result.text)
```

### Custom Categories

```lua
local custom_categories = {
    {
        name = "Invoices",
        extensions = {".pdf"},
        keywords = {"invoice", "payment", "receipt"},
        destination = "Documents/Invoices"
    },
    {
        name = "Contracts",
        extensions = {".pdf", ".docx"},
        keywords = {"contract", "agreement", "terms"},
        destination = "Documents/Legal"
    }
}

-- Note: Custom categories support coming in future update
-- Currently use category_preset parameter
```

### Automated Workflow

```lua
-- Weekly download cleanup script
local function cleanup_downloads()
    -- 1. Analyze what's there
    local analysis = Template.execute("file-classification", {
        source_path = os.getenv("HOME") .. "/Downloads",
        category_preset = "downloads",
        action = "report",
        output_format = "json"
    })

    print("Found " .. analysis.metadata.files_scanned .. " files")

    -- 2. Move files to organized folders
    local result = Template.execute("file-classification", {
        source_path = os.getenv("HOME") .. "/Downloads",
        category_preset = "downloads",
        action = "move",
        destination_base = os.getenv("HOME") .. "/Downloads/Organized",
        dry_run = false  -- Only after testing!
    })

    print("Classification complete!")
end

cleanup_downloads()
```

## Troubleshooting

### Issue: "Source path does not exist"

**Cause:** Invalid or non-existent file path.

**Solution:**
```bash
# Check path exists
ls -la ~/path/to/directory

# Use absolute paths
llmspell template exec file-classification \
  --param source_path=/Users/yourname/Downloads
```

### Issue: "Permission denied"

**Cause:** Insufficient permissions to read source or write destination.

**Solution:**
```bash
# Check permissions
ls -la ~/path/to/directory

# Fix permissions
chmod u+rw ~/path/to/directory/*

# Or use sudo (not recommended for personal files)
```

### Issue: Files not moving in dry-run mode

**Cause:** Dry-run mode is enabled (this is expected behavior).

**Solution:**
```bash
# Dry-run shows preview only - this is correct!
# To actually move files:
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param dry_run=false  # Only after reviewing dry-run output!
```

### Issue: All files classified as "Uncategorized"

**Cause:** File extensions don't match any categories in preset.

**Solution:**
```bash
# 1. Try different preset
--param category_preset=media  # Instead of documents

# 2. Try hybrid strategy (adds content matching)
--param classification_strategy=hybrid

# 3. Check what files you have
ls ~/path/to/directory
```

### Issue: "No categories defined"

**Cause:** Invalid category_preset value.

**Solution:**
```bash
# Use valid preset names:
--param category_preset=documents  # Valid
--param category_preset=media      # Valid
--param category_preset=code       # Valid
--param category_preset=downloads  # Valid

# Not: custom_categories (not supported yet)
```

### Issue: Slow performance on large directories

**Cause:** Content-based strategy or large file counts.

**Solution:**
```bash
# Use faster extension strategy
--param classification_strategy=extension

# Process in batches (manual approach)
llmspell template exec file-classification \
  --param source_path=~/Downloads/Batch1

# Disable recursive mode
--param recursive=false
```

## Best Practices

### 1. Always Dry-Run First

```bash
# GOOD: Preview before executing
--param dry_run=true  # Review output
--param dry_run=false # Execute after review

# BAD: Direct execution without preview
--param dry_run=false # Risky without preview!
```

### 2. Use Specific Presets

```bash
# GOOD: Specific preset for file types
--param category_preset=media  # For photos/videos

# SUBOPTIMAL: Generic preset for mixed types
--param category_preset=documents  # Won't classify media files
```

### 3. Backup Before Mass Operations

```bash
# GOOD: Backup first
cp -r ~/Downloads ~/Downloads.backup

# Then classify
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param dry_run=false
```

### 4. Test on Small Directory First

```bash
# GOOD: Test on subset
mkdir ~/test_classification
cp ~/Downloads/*.pdf ~/test_classification/
llmspell template exec file-classification \
  --param source_path=~/test_classification

# Then apply to full directory
```

### 5. Use JSON Output for Large Operations

```bash
# GOOD: JSON for automation
llmspell template exec file-classification \
  --param output_format=json > report.json

# Parse with jq
cat report.json | jq '.total_files'
cat report.json | jq '.categories'
```

### 6. Combine with Other Tools

```bash
# Find files modified in last 7 days, then classify
find ~/Downloads -type f -mtime -7 -exec \
  llmspell template exec file-classification \
  --param source_path={} ;
```

## Safety Features

### 1. Dry-Run Default

Template defaults to `dry_run=true` to prevent accidents.

### 2. Path Validation

Validates source path exists and is accessible before classification.

### 3. Error Handling

- Permission errors: Logged, continue with remaining files
- Disk space: Checked before move operations
- Invalid paths: Clear error messages

### 4. Progress Visibility

Real-time progress updates for long operations (>10 files).

### 5. Confidence Scores

Classification confidence (0.0-1.0) helps identify uncertain classifications.

## Performance

### Extension Strategy
- **Speed**: < 1ms per file
- **Use Case**: 1000+ files, known extensions
- **Bottleneck**: None (file metadata only)

### Content Strategy
- **Speed**: ~5-10ms per file (text files)
- **Use Case**: < 100 files, unknown extensions
- **Bottleneck**: File I/O

### Hybrid Strategy
- **Speed**: ~1-2ms per file (mostly extension matches)
- **Use Case**: Mixed scenarios
- **Bottleneck**: File I/O for content fallback

### Benchmark Results

```
100 files (extension): ~0.1 seconds
100 files (content):    ~1.0 seconds
100 files (hybrid):     ~0.2 seconds

1000 files (extension): ~1.0 seconds
1000 files (hybrid):    ~1.5 seconds
```

## Common Patterns

### Pattern 1: Weekly Download Cleanup

```bash
#!/bin/bash
# cleanup_downloads.sh

# Dry-run to review
llmspell template exec file-classification \
  --param source_path=~/Downloads \
  --param category_preset=downloads \
  --param action=move \
  --param destination_base=~/Downloads/Organized \
  --param dry_run=true

# Uncomment to execute:
# --param dry_run=false
```

### Pattern 2: Photo Organization

```bash
# Organize photos by moving to dated folders
llmspell template exec file-classification \
  --param source_path=~/Pictures/Import \
  --param category_preset=media \
  --param recursive=true \
  --param action=move \
  --param destination_base=~/Pictures/Library
```

### Pattern 3: Code Archive

```bash
# Create language-organized code archive
llmspell template exec file-classification \
  --param source_path=~/Projects \
  --param category_preset=code \
  --param recursive=true \
  --param action=copy \
  --param destination_base=~/Archive/Code
```

## Next Steps

- Explore [Workflow Orchestrator](workflow-orchestrator.md) for multi-template pipelines
- See [Interactive Chat](interactive-chat.md) for conversational file management
- Check [Document Processor](document-processor.md) for PDF organization
- Review [Template Overview](README.md) for all available templates

## Version History

### 0.1.0 (Current)
- Initial release
- 3 classification strategies (extension, content, hybrid)
- 4 category presets (documents, media, code, downloads)
- Dry-run mode with safe defaults
- Multiple output formats (text, markdown, JSON)
- Recursive directory scanning
- Progress reporting for bulk operations
