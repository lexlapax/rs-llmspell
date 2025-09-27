# IDE Integration Guide

**Version**: 0.9.0
**Last Updated**: December 2024

> **🔌 IDE Integration**: Connect LLMSpell kernel with VS Code, Jupyter Lab, vim/neovim, and other development environments.

**🔗 Navigation**: [← User Guide](README.md) | [Service Deployment →](service-deployment.md) | [API Reference →](api/README.md)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Kernel Connection Basics](#kernel-connection-basics)
3. [VS Code Setup](#vs-code-setup)
4. [Jupyter Lab Integration](#jupyter-lab-integration)
5. [vim/neovim Setup](#vimneovim-setup)
6. [Connection File Format](#connection-file-format)
7. [Debug Adapter Protocol (DAP)](#debug-adapter-protocol-dap)
8. [Common Workflows](#common-workflows)
9. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Start LLMSpell Kernel

```bash
# Start kernel with connection file for IDE discovery
./target/release/llmspell kernel start --connection-file ~/.llmspell/kernel.json

# Start in background with specific port
./target/release/llmspell kernel start --daemon --port 9555 --connection-file ~/.llmspell/kernel.json

# Check kernel is running
./target/release/llmspell kernel status
```

### Connect from IDE

**VS Code**: Install Jupyter extension, use "Connect to Existing Jupyter Server"
**Jupyter Lab**: Use connection file with `--existing` flag
**vim/neovim**: Configure LSP client with kernel connection

## Kernel Connection Basics

LLMSpell kernel implements the Jupyter protocol with 5 ZeroMQ channels:
- **Shell**: Execute requests and replies
- **IOPub**: Broadcast outputs and status
- **Stdin**: Input requests (prompts)
- **Control**: Control commands (shutdown, interrupt)
- **Heartbeat**: Connection liveness monitoring

The kernel can run in two modes:
1. **Embedded**: Spawns in background thread (default)
2. **External**: Connects to standalone kernel process

## VS Code Setup

### Prerequisites

1. Install VS Code extensions:
   - **Jupyter** (Microsoft)
   - **Python** (Microsoft) - for notebook support
   - Optional: **CodeLLDB** for debugging

### Connect to LLMSpell Kernel

#### Method 1: Using Connection File

1. Start LLMSpell kernel:
```bash
./target/release/llmspell kernel start \
  --connection-file ~/.llmspell/kernel.json \
  --daemon
```

2. In VS Code:
   - Open Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`)
   - Run: "Jupyter: Specify Jupyter Server for Connections"
   - Select: "Existing: Specify the URI of an existing server"
   - Enter the connection file path

3. Create/Open a notebook:
   - Create new file with `.ipynb` extension
   - Select "LLMSpell" as kernel

#### Method 2: Manual Connection

1. Get connection info:
```bash
cat ~/.llmspell/kernel.json
```

2. In VS Code settings.json:
```json
{
  "jupyter.kernels.trusted": [
    "/path/to/.llmspell/kernel.json"
  ],
  "jupyter.jupyterServerType": "remote",
  "jupyter.jupyterServer.uriList": [
    {
      "name": "LLMSpell Kernel",
      "uri": "http://localhost:9555/?token=your-token"
    }
  ]
}
```

### Debug Adapter Protocol (DAP) Setup

1. Create `.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "llmspell",
      "request": "attach",
      "name": "Attach to LLMSpell",
      "host": "localhost",
      "port": 9556,
      "pathMappings": [
        {
          "localRoot": "${workspaceFolder}",
          "remoteRoot": "/workspace"
        }
      ]
    }
  ]
}
```

2. Start debugging:
   - Set breakpoints in Lua/JavaScript files
   - Press F5 to start debugging
   - Use debug console for REPL

### Features in VS Code

- **IntelliSense**: Code completion for Lua/JavaScript
- **Variable Explorer**: View variables in notebook interface
- **Debugging**: Breakpoints, stepping, watch expressions
- **Output Streaming**: Real-time execution output
- **Markdown Support**: Rich text cells in notebooks

## Jupyter Lab Integration

### Installation

```bash
# Install Jupyter Lab if needed
pip install jupyterlab

# Optional: Install kernel spec globally
mkdir -p ~/.local/share/jupyter/kernels/llmspell
```

### Create Kernel Spec

Create `~/.local/share/jupyter/kernels/llmspell/kernel.json`:

```json
{
  "display_name": "LLMSpell",
  "language": "lua",
  "argv": [
    "/usr/local/bin/llmspell",
    "kernel",
    "start",
    "--connection-file",
    "{connection_file}"
  ],
  "metadata": {
    "debugger": true
  }
}
```

### Connect to Running Kernel

```bash
# Start LLMSpell kernel
./target/release/llmspell kernel start \
  --connection-file ~/.llmspell/kernel.json \
  --daemon

# Connect Jupyter Lab
jupyter lab --existing ~/.llmspell/kernel.json
```

### Use in Notebooks

```python
# In Jupyter notebook cell (Lua syntax)
%%lua
-- LLMSpell Lua code
local agent = Agent.builder()
  :model("openai/gpt-4o-mini")
  :build()

local result = agent:execute({
  prompt = "Explain quantum computing"
})

print(result.content)
```

### Features in Jupyter Lab

- **Multi-language Support**: Lua, JavaScript, Python (via bridge)
- **Rich Output**: HTML, images, plots
- **Interactive Widgets**: ipywidgets compatible
- **Variable Inspector**: Built-in variable explorer
- **Code Completion**: Tab completion support
- **Magic Commands**: Custom %%lua, %%js magics

## vim/neovim Setup

### LSP Configuration (Neovim)

1. Install required plugins:
```vim
" Using packer.nvim
use 'neovim/nvim-lspconfig'
use 'hrsh7th/nvim-cmp'
use 'hrsh7th/cmp-nvim-lsp'
use 'L3MON4D3/LuaSnip'
```

2. Configure LLMSpell LSP in `init.lua`:
```lua
local lspconfig = require('lspconfig')

-- Custom LLMSpell LSP config
lspconfig.llmspell = {
  default_config = {
    cmd = {
      'llmspell', 'kernel', 'start',
      '--lsp',
      '--port', '9557'
    },
    filetypes = {'lua', 'javascript'},
    root_dir = lspconfig.util.root_pattern('.llmspell', '.git'),
    settings = {
      llmspell = {
        enableSnippets = true,
        enableDebug = true
      }
    }
  }
}

lspconfig.llmspell.setup{
  on_attach = function(client, bufnr)
    -- Enable completion
    vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')

    -- Keybindings
    local opts = { noremap=true, silent=true, buffer=bufnr }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
  end,
  capabilities = require('cmp_nvim_lsp').default_capabilities()
}
```

### REPL Integration

1. Install vim-slime or similar:
```vim
use 'jpalardy/vim-slime'
```

2. Configure REPL connection:
```vim
" .vimrc or init.vim
let g:slime_target = "tmux"
let g:slime_default_config = {
  \ 'socket_name': get(split($TMUX, ','), 0),
  \ 'target_pane': '{last}'
\ }

" Custom command for LLMSpell REPL
command! LLMSpellREPL :terminal llmspell repl --kernel localhost:9555
```

3. Keybindings for REPL:
```vim
" Send selection to REPL
xmap <leader>s <Plug>SlimeRegionSend
" Send paragraph to REPL
nmap <leader>s <Plug>SlimeParagraphSend
" Send cell to REPL
nmap <leader>c <Plug>SlimeConfig
```

### DAP Debugging (Neovim)

1. Install nvim-dap:
```vim
use 'mfussenegger/nvim-dap'
use 'rcarriga/nvim-dap-ui'
```

2. Configure DAP for LLMSpell:
```lua
local dap = require('dap')

dap.adapters.llmspell = {
  type = 'server',
  host = 'localhost',
  port = 9556
}

dap.configurations.lua = {
  {
    type = 'llmspell',
    request = 'launch',
    name = 'Launch LLMSpell Script',
    program = '${file}',
    cwd = '${workspaceFolder}',
    stopOnEntry = false
  }
}
```

## Connection File Format

The kernel creates a connection file with ZeroMQ endpoints:

```json
{
  "transport": "tcp",
  "ip": "127.0.0.1",
  "shell_port": 50510,
  "iopub_port": 50511,
  "stdin_port": 50512,
  "control_port": 50513,
  "hb_port": 50514,
  "key": "a0b1c2d3-e4f5-6789-abcd-ef0123456789",
  "signature_scheme": "hmac-sha256",
  "kernel_name": "llmspell"
}
```

### Security

- **HMAC Authentication**: Messages signed with `key`
- **Local Only**: Default binding to localhost
- **Port Range**: Configurable port allocation
- **Token Auth**: Optional token for HTTP endpoints

## Debug Adapter Protocol (DAP)

LLMSpell implements DAP for IDE debugging integration:

### Supported Features

- **Breakpoints**: Line and conditional
- **Stepping**: Step in/over/out, continue
- **Variables**: Locals, globals, watch expressions
- **Call Stack**: Full stack trace
- **REPL**: Debug console evaluation
- **Source Maps**: Accurate file:line mapping

### DAP Commands

```lua
-- In debug console
:break 10           -- Set breakpoint at line 10
:watch myVar        -- Add watch expression
:locals             -- Show local variables
:stack              -- Show call stack
:continue           -- Resume execution
:step               -- Step into
:next               -- Step over
:finish             -- Step out
```

## Common Workflows

### Interactive Development

1. Start kernel in development mode:
```bash
./target/release/llmspell kernel start \
  --connection-file ~/.llmspell/kernel.json \
  --idle-timeout 0 \
  --trace debug
```

2. Connect IDE of choice
3. Create notebook or script
4. Iterate with hot-reload:
```lua
-- Reload modules
package.loaded['mymodule'] = nil
local mymodule = require('mymodule')
```

### Production Debugging

1. Connect to production kernel:
```bash
# Get connection info from production
ssh prod-server cat /var/lib/llmspell/kernel.json > local-kernel.json

# Connect locally
./target/release/llmspell kernel connect --connection-file local-kernel.json
```

2. Attach debugger:
```bash
# In IDE, use remote debugging with SSH tunneling
ssh -L 9556:localhost:9556 prod-server
```

### Multi-Client Collaboration

1. Start kernel with high client limit:
```bash
./target/release/llmspell kernel start \
  --max-clients 50 \
  --connection-file /shared/kernel.json
```

2. Share connection file with team
3. Multiple developers connect simultaneously
4. Shared state and debugging sessions

## Troubleshooting

### Connection Issues

**Issue**: "Could not connect to kernel"
```bash
# Check kernel is running
./target/release/llmspell kernel status

# Check ports are open
lsof -i :9555

# Test ZeroMQ connectivity
nc -zv localhost 50510-50514
```

**Issue**: "Authentication failed"
```bash
# Verify HMAC key matches
cat ~/.llmspell/kernel.json | jq .key

# Regenerate connection file
rm ~/.llmspell/kernel.json
./target/release/llmspell kernel start --connection-file ~/.llmspell/kernel.json
```

### VS Code Issues

**Issue**: "No kernel specs found"
```bash
# Install kernel spec manually
jupyter kernelspec install --user ~/.local/share/jupyter/kernels/llmspell

# List installed kernels
jupyter kernelspec list
```

**Issue**: "IntelliSense not working"
```json
// settings.json
{
  "jupyter.enableExtendedKernelCompletions": true,
  "jupyter.enableCellCodeLens": true
}
```

### Jupyter Lab Issues

**Issue**: "Kernel keeps disconnecting"
```bash
# Increase timeout
./target/release/llmspell kernel start --idle-timeout 0

# Check for memory issues
top -p $(pgrep llmspell)
```

**Issue**: "Variables not showing"
```python
# Enable variable inspector
%config InlineBackend.figure_format = 'retina'
%load_ext autoreload
%autoreload 2
```

### vim/neovim Issues

**Issue**: "LSP not attaching"
```vim
:LspInfo  " Check LSP status
:LspLog   " View LSP logs
:checkhealth lsp  " Diagnose issues
```

**Issue**: "Completion not working"
```lua
-- Check capabilities
:lua print(vim.inspect(vim.lsp.get_active_clients()))

-- Force completion
:lua vim.lsp.buf.completion()
```

### Performance Tips

1. **Use connection pooling**: Reuse kernel connections
2. **Enable caching**: Cache frequently used completions
3. **Limit concurrent operations**: Set max-clients appropriately
4. **Monitor resources**: Watch memory/CPU usage
5. **Use local kernels**: Reduce network latency

---

**🔗 Next Steps**: [API Reference →](api/README.md) | [Examples →](../../examples/README.md)