# File Organizer v1.0 (Universal Layer)

A simple AI-powered file organization system that solves the universal problem: "My files are a complete mess!" Perfect for anyone who struggles with file chaos and wants immediate organization help.

## Overview

The File Organizer demonstrates:
- **Simple Sequential Workflow**: Scan → Classify → Organize (3 clear steps)
- **3 Simple Agents**: Reduced from complex 8-agent system for universal appeal
- **Immediate Results**: Organization plan in under 5 minutes
- **No Technical Knowledge**: Just run and get organized folders
- **Universal Problem**: File chaos that every computer user experiences

## Universal Appeal Features

### Problem: "My files are a complete mess"
Everyone with a computer faces this - downloads folder chaos, desktop clutter, scattered documents. This app solves it simply.

### Solution: 3-Step AI Organization
1. **Scan**: AI examines your files to understand content
2. **Classify**: Files sorted into clear categories (Documents, Images, Videos, etc.)
3. **Organize**: Folder structure suggested with file placement plan

### Target Users
- Any computer user with file organization problems
- Students with assignment chaos
- Parents with family photo/document clutter  
- Remote workers with scattered project files
- Anyone who feels overwhelmed by digital mess

## Quick Start

### 1. Basic Run (No API Keys)
```bash
./target/debug/llmspell run examples/script-users/applications/file-organizer/main.lua
```

### 2. With Configuration
```bash
./target/debug/llmspell -c examples/script-users/applications/file-organizer/config.toml run examples/script-users/applications/file-organizer/main.lua
```

### 3. Debug Mode
```bash
./target/debug/llmspell --debug run examples/script-users/applications/file-organizer/main.lua
```

## Simple Architecture

### 3 Simple Agents (Universal Complexity)

| Agent | Purpose | What It Does |
|-------|---------|--------------|
| **File Scanner** | Content Analysis | Looks at file names and types to understand what you have |
| **Category Classifier** | Smart Sorting | Puts files into clear categories everyone understands |
| **Organization Suggester** | Structure Planning | Creates simple folder structure that makes sense |

### Simple Workflow
```
Sequential: scan → classify → organize
├── Scan Files (Tool: file_operations)
├── Classify Files (Agent: category_classifier) 
└── Suggest Organization (Agent: organization_suggester)
```

### File Categories
- **Documents**: PDFs, Word docs, text files, presentations
- **Images**: Photos, screenshots, graphics
- **Videos**: Video files, tutorials, recordings
- **Audio**: Music, recordings, podcasts
- **Code**: Programming files, scripts
- **Archive**: ZIP files, backups
- **Other**: Everything else

## Sample Results

### Before: Messy Files
```
/tmp/messy_files/
├── vacation_photo_2023.jpg
├── important_document.pdf
├── random_notes.txt
├── project_code.py
├── music_track.mp3
├── presentation_draft.pptx
├── backup_archive.zip
├── screenshot_20240822.png
├── meeting_notes_Q3.docx
└── video_tutorial.mp4
```

### After: Organized Structure
```
/tmp/organized_files/
├── Documents/
│   ├── important_document.pdf
│   ├── random_notes.txt
│   ├── presentation_draft.pptx
│   └── meeting_notes_Q3.docx
├── Images/
│   ├── vacation_photo_2023.jpg
│   └── screenshot_20240822.png
├── Videos/
│   └── video_tutorial.mp4
├── Audio/
│   └── music_track.mp3
├── Code/
│   └── project_code.py
└── Archive/
    └── backup_archive.zip
```

## Universal Appeal Validation

### Success Metrics
- ✅ **<5 minutes to value**: Organization plan created quickly
- ✅ **>80% task completion**: Simple enough for anyone
- ✅ **>70% recommendation likelihood**: Solves real pain point
- ✅ **<10% complexity abandonment**: No technical barriers

### Why Universal?
1. **Real Problem**: Everyone has file chaos
2. **Simple Solution**: 3 clear steps 
3. **Immediate Value**: See results right away
4. **No Learning**: Just run and get organized
5. **Clear Categories**: Folders everyone understands

## Technical Architecture (Simplified)

### Crates Used (Core Only)
- `llmspell-core`: Basic agent and workflow types
- `llmspell-agents`: Simple agent creation
- `llmspell-bridge`: Basic Lua integration

### Tools Used (Essential Only)
- `file_operations`: List files, create directories, write plans
- `text_manipulator`: Basic text processing (minimal use)

### State Management: REMOVED
- No `State.get()` patterns (too complex for universal users)
- Direct result access only: `result.outputs.step_name`
- Immediate feedback, no complex persistence

### Workflow Complexity: MINIMAL
- Single sequential workflow (scan → classify → organize)
- No parallel, loop, or conditional complexity
- Linear progression everyone can follow

## Configuration

### Customization Options

Edit `main.lua` for basic settings:
```lua
local config = {
    settings = {
        max_files = 20,  -- Process more files
        categories = {"Documents", "Images", "Videos", "Audio", "Code", "Archive", "Other"}
    }
}
```

### Directory Settings
```lua
files = {
    target_directory = "/your/messy/folder/",      -- Where your mess is
    organized_directory = "/your/organized/folder/", -- Where it goes  
    organization_plan = "/tmp/organization-plan.txt" -- The plan file
}
```

## Output Files

| File | Description |
|------|-------------|
| `/tmp/messy_files/` | Sample messy files (demo) |
| `/tmp/organized_files/` | Organized folder structure |
| `/tmp/organization-plan.txt` | Step-by-step organization guide |

## Progression Path

### Natural Learning Progression
1. **Start Here (Universal)**: File Organizer - solve file chaos
2. **Next Step (Power User)**: Content Creator - add conditional workflows
3. **Advanced (Business)**: Communication Manager - add state persistence
4. **Expert (Professional)**: Process Orchestrator - full automation

### Bridge to Power User Layer
Users who complete file organization naturally want:
- More sophisticated categorization (conditional logic)
- Content creation workflows
- Project organization systems
- Quality control decisions

## Common Use Cases

### Home Users
- Organize family photos scattered across desktop
- Sort downloaded files into proper folders
- Clean up document chaos before important events

### Students  
- Organize assignment files by subject/semester
- Sort research materials for projects
- Clean up downloads folder for better productivity

### Remote Workers
- Organize work files across projects
- Sort meeting recordings and notes
- Clean up desktop for better focus

### Small Business Owners
- Organize client files and documents
- Sort business records and receipts
- Prepare file systems for tax season

## Troubleshooting

### "No files found"
- Check that `/tmp/messy_files/` exists
- Verify file permissions
- Sample files are created automatically

### "Agent needs API key"
- System continues with basic organization
- Set API keys for AI-powered categorization
- Manual categorization suggestions provided

### Organization not perfect
- AI suggestions are starting points
- Adjust categories based on your needs
- Move files manually for perfect fit

## Cost Considerations

**Very Low Cost**: Universal layer optimized for affordability
- File scanning: ~$0.001 per 10 files
- Categorization: ~$0.002 per 10 files  
- Organization planning: ~$0.001 per plan
- **Typical run cost**: $0.01 or less

## Related Applications

### Other Universal Layer Apps
- **Research Collector**: Simple information gathering
- Coming soon: More universal solutions

### Progression Apps
- **Content Creator** (Power User): Conditional workflows
- **Communication Manager** (Business): State persistence
- **Process Orchestrator** (Professional): Full automation

## Extension Ideas

### Stay Universal (Don't Add Complexity)
- More file type recognition
- Better category suggestions
- Simpler folder naming
- Integration with cloud storage

### Avoid These (Too Complex for Universal)
- Advanced filtering rules
- Custom taxonomies
- Automated file moving
- Complex state management

## Support

For issues or questions:
- Keep it simple - this is the entry point
- Focus on universal problems only
- Check progression apps for advanced features
- Universal appeal is the key success metric