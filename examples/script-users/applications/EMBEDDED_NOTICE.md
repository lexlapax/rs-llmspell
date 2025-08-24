# Important: Applications Embedded in CLI Binary

## These applications have been copied to `llmspell-cli/resources/applications/`

For single binary distribution, all 7 example applications are now embedded directly in the `llmspell` binary. The copies in this directory (`examples/script-users/applications/`) remain for:

1. **Development and testing** - Easier to edit and test here
2. **Documentation** - Reference implementation with full comments
3. **Traditional usage** - Can still be run with `llmspell run <path>`

## Embedded Versions

The applications in `llmspell-cli/resources/applications/` are:
- Compiled into the binary using `include_str!`
- Accessible via `llmspell apps` commands
- Extract to temp directory at runtime
- Zero path configuration required

## Usage

### Single Binary (Recommended)
```bash
llmspell apps list                # List all embedded apps
llmspell apps file-organizer      # Run embedded file organizer
llmspell apps research-collector  # Run embedded research collector
```

### Traditional Path-Based
```bash
llmspell run examples/script-users/applications/file-organizer/main.lua
```

## Synchronization

If you modify applications here for development:
1. Test your changes locally
2. Copy updated files to `llmspell-cli/resources/applications/`
3. Rebuild the CLI to embed the new versions

## Why Two Locations?

- **Here (`examples/`)**: Development, testing, documentation
- **CLI (`resources/`)**: Embedded for single binary distribution

This dual approach ensures both developer convenience and end-user simplicity.