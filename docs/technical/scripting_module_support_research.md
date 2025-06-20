# Scripting Language Module Support Research

## Overview

This document explores extending rs-llmspell beyond embedded scripting to provide native module support for Lua, JavaScript, and other scripting languages. Instead of only embedding runtimes within rs-llmspell, this approach enables rs-llmspell functionality to be imported as modules into existing scripting environments.

## Concept: Bidirectional Integration

### Current Architecture (Embedded Only)
```
┌─────────────────────────────────────┐
│         Rs-LLMSpell (Rust)          │
│  ┌─────────────┐  ┌─────────────┐   │
│  │ Lua Runtime │  │ JS Runtime  │   │
│  │   (mlua)    │  │    (boa)    │   │
│  └─────────────┘  └─────────────┘   │
│                                     │
│  ┌─────────────────────────────────┐ │
│  │     Core Functionality          │ │
│  │  (Agents, Tools, Workflows)     │ │
│  └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

### Proposed Bidirectional Architecture
```
┌─────────────────────────────────────┐    ┌─────────────────────────────────────┐
│         Rs-LLMSpell (Rust)          │    │      External Lua Runtime           │
│  ┌─────────────┐  ┌─────────────┐   │    │  ┌─────────────────────────────────┐ │
│  │ Lua Runtime │  │ JS Runtime  │   │    │  │    require("llmspell")          │ │
│  │   (mlua)    │  │    (boa)    │   │    │  │                                 │ │
│  └─────────────┘  └─────────────┘   │    │  │  local agent = llmspell.Agent() │ │
│                                     │    │  │  local result = agent:chat(msg) │ │
│  ┌─────────────────────────────────┐ │    │  └─────────────────────────────────┘ │
│  │     Core Functionality          │ │◄──►│                                     │
│  │  (Agents, Tools, Workflows)     │ │    │         External Node.js            │
│  └─────────────────────────────────┘ │    │  ┌─────────────────────────────────┐ │
│                                     │    │  │const llmspell = require('@rs/  │ │
│  ┌─────────────────────────────────┐ │    │  │   llmspell')                    │ │
│  │      Module Interface           │ │    │  │                                 │ │
│  │  (C API, WASM, Native Modules)  │ │    │  │const agent = new llmspell.     │ │
│  └─────────────────────────────────┘ │    │  │   Agent()                       │ │
└─────────────────────────────────────┘    │  └─────────────────────────────────┘ │
                                           └─────────────────────────────────────┘
```

## Module Integration Concepts

### 1. Native Module Approach

**Concept**: Compile rs-llmspell as native modules that can be dynamically loaded by scripting runtimes.

**Benefits**:
- Near-native performance
- Full feature access
- Memory sharing between Rust and script runtime
- No serialization overhead

**Challenges**:
- Platform-specific compilation
- Runtime version compatibility
- Memory management complexity
- Debugging across language boundaries

### 2. C API Bridge Approach

**Concept**: Expose rs-llmspell functionality through a stable C API that scripting languages can bind to.

**Benefits**:
- Language agnostic
- Stable ABI
- Wide compatibility
- Easier distribution

**Challenges**:
- Limited expressiveness of C API
- Manual memory management
- Error handling complexity
- Performance overhead

### 3. WebAssembly (WASM) Module Approach

**Concept**: Compile rs-llmspell to WebAssembly for universal module compatibility.

**Benefits**:
- Universal compatibility
- Sandboxed execution
- Web and server compatibility
- Easy distribution

**Challenges**:
- Performance limitations
- Limited system access
- WASM-specific constraints
- Async execution complexity

### 4. Inter-Process Communication (IPC) Approach

**Concept**: Run rs-llmspell as a separate process and communicate via IPC mechanisms.

**Benefits**:
- Language independence
- Process isolation
- Fault tolerance
- Scalability

**Challenges**:
- Communication overhead
- Process management complexity
- Serialization costs
- Latency considerations

## Lua Module Integration Research

### Lua C Module Implementation

**Architecture**: Implement rs-llmspell as a Lua C module using the Lua C API.

```c
// llmspell_lua.c - C module wrapper
#include <lua.h>
#include <lauxlib.h>
#include <lualib.h>
#include "llmspell_c_api.h"

// Agent userdata type
typedef struct {
    LLMSpellAgent* agent;
    int is_valid;
} AgentUserData;

// Create new agent
static int lua_agent_new(lua_State* L) {
    // Get configuration from Lua table
    luaL_checktype(L, 1, LUA_TTABLE);
    
    // Extract configuration
    lua_getfield(L, 1, "system_prompt");
    const char* system_prompt = luaL_checkstring(L, -1);
    lua_pop(L, 1);
    
    lua_getfield(L, 1, "provider");
    const char* provider = luaL_optstring(L, -1, "openai");
    lua_pop(L, 1);
    
    // Create agent configuration
    LLMSpellAgentConfig config = {
        .system_prompt = system_prompt,
        .provider = provider,
        .tools = NULL,
        .tool_count = 0
    };
    
    // Create agent instance
    LLMSpellAgent* agent = llmspell_agent_create(&config);
    if (!agent) {
        return luaL_error(L, "Failed to create agent");
    }
    
    // Create userdata
    AgentUserData* ud = (AgentUserData*)lua_newuserdata(L, sizeof(AgentUserData));
    ud->agent = agent;
    ud->is_valid = 1;
    
    // Set metatable
    luaL_getmetatable(L, "LLMSpell.Agent");
    lua_setmetatable(L, -2);
    
    return 1;
}

// Agent chat method
static int lua_agent_chat(lua_State* L) {
    AgentUserData* ud = (AgentUserData*)luaL_checkudata(L, 1, "LLMSpell.Agent");
    const char* message = luaL_checkstring(L, 2);
    
    if (!ud->is_valid) {
        return luaL_error(L, "Agent has been destroyed");
    }
    
    // Execute chat
    LLMSpellResult result = llmspell_agent_chat(ud->agent, message);
    
    if (result.error) {
        lua_pushnil(L);
        lua_pushstring(L, result.error_message);
        return 2;
    }
    
    lua_pushstring(L, result.response);
    return 1;
}

// Agent garbage collection
static int lua_agent_gc(lua_State* L) {
    AgentUserData* ud = (AgentUserData*)luaL_checkudata(L, 1, "LLMSpell.Agent");
    
    if (ud->is_valid) {
        llmspell_agent_destroy(ud->agent);
        ud->is_valid = 0;
    }
    
    return 0;
}

// Agent metatable
static const luaL_Reg agent_methods[] = {
    {"chat", lua_agent_chat},
    {"execute", lua_agent_execute},
    {"add_tool", lua_agent_add_tool},
    {"get_state", lua_agent_get_state},
    {"__gc", lua_agent_gc},
    {NULL, NULL}
};

// Module functions
static const luaL_Reg llmspell_functions[] = {
    {"agent", lua_agent_new},
    {"create_tool", lua_create_tool},
    {"create_workflow", lua_create_workflow},
    {"version", lua_get_version},
    {NULL, NULL}
};

// Module initialization
LUAMOD_API int luaopen_llmspell(lua_State* L) {
    // Initialize rs-llmspell library
    if (!llmspell_init()) {
        return luaL_error(L, "Failed to initialize llmspell");
    }
    
    // Create agent metatable
    luaL_newmetatable(L, "LLMSpell.Agent");
    lua_pushvalue(L, -1);
    lua_setfield(L, -2, "__index");
    luaL_setfuncs(L, agent_methods, 0);
    
    // Create module table
    luaL_newlib(L, llmspell_functions);
    
    return 1;
}
```

### Lua Module Usage Examples

```lua
-- Direct Lua usage (external Lua runtime)
local llmspell = require("llmspell")

-- Create agent with configuration
local agent = llmspell.agent({
    system_prompt = "You are a helpful assistant",
    provider = "openai",
    model = "gpt-4",
    tools = {
        llmspell.create_tool({
            name = "calculator",
            description = "Perform mathematical calculations",
            schema = {
                type = "object",
                properties = {
                    expression = {type = "string"}
                }
            },
            execute = function(params)
                -- Lua calculation logic
                return load("return " .. params.expression)()
            end
        })
    }
})

-- Use agent
local response = agent:chat("What is 15 * 23?")
print("Agent response:", response)

-- Complex workflow example
local workflow = llmspell.create_workflow({
    type = "sequential",
    steps = {
        {
            name = "research",
            agent = research_agent,
            input = "{{topic}}"
        },
        {
            name = "analyze", 
            agent = analysis_agent,
            input = "{{research.output}}"
        },
        {
            name = "report",
            agent = report_agent,
            input = {
                research = "{{research.output}}",
                analysis = "{{analyze.output}}"
            }
        }
    }
})

local result = workflow:execute({topic = "AI trends 2025"})
print("Workflow result:", result.final_output)
```

### LuaRocks Package Distribution

```lua
-- llmspell-dev-1.rockspec
package = "llmspell"
version = "dev-1"
source = {
   url = "git+https://github.com/company/rs-llmspell.git"
}
description = {
   summary = "Scriptable LLM interactions for Lua",
   detailed = [[
      Rs-LLMSpell provides powerful agent orchestration, tool integration,
      and workflow management capabilities directly in Lua applications.
   ]],
   homepage = "https://github.com/company/rs-llmspell",
   license = "MIT"
}
dependencies = {
   "lua >= 5.1"
}
external_dependencies = {
   LLMSPELL = {
      header = "llmspell.h",
      library = "llmspell"
   }
}
build = {
   type = "builtin",
   modules = {
      llmspell = {
         sources = {"src/llmspell_lua.c"},
         libraries = {"llmspell"},
         incdirs = {"$(LLMSPELL_INCDIR)"},
         libdirs = {"$(LLMSPELL_LIBDIR)"}
      }
   }
}
```

### Lua Installation and Usage

```bash
# Install via LuaRocks
luarocks install llmspell

# Or build from source
git clone https://github.com/company/rs-llmspell
cd rs-llmspell/bindings/lua
luarocks make
```

```lua
-- Use in existing Lua applications
local llmspell = require("llmspell")
local json = require("json")

-- OpenResty/nginx integration example
local function handle_chat_request()
    local ngx = require("ngx")
    local cjson = require("cjson")
    
    -- Parse request
    ngx.req.read_body()
    local body = ngx.req.get_body_data()
    local request = cjson.decode(body)
    
    -- Create agent
    local agent = llmspell.agent({
        system_prompt = "You are a customer service assistant",
        provider = "anthropic",
        tools = {customer_lookup_tool, ticket_system_tool}
    })
    
    -- Process request
    local response = agent:chat(request.message)
    
    -- Return response
    ngx.header.content_type = "application/json"
    ngx.say(cjson.encode({response = response}))
end

return {
    handle_chat_request = handle_chat_request
}
```

## JavaScript Module Integration Research

### Node.js Native Module Implementation

**Architecture**: Implement rs-llmspell as a Node.js native addon using N-API.

```cpp
// llmspell_node.cpp - Node.js native addon
#include <napi.h>
#include "llmspell_c_api.h"
#include <memory>

class AgentWrapper : public Napi::ObjectWrap<AgentWrapper> {
public:
    static Napi::Object Init(Napi::Env env, Napi::Object exports);
    AgentWrapper(const Napi::CallbackInfo& info);
    ~AgentWrapper();

private:
    static Napi::FunctionReference constructor;
    
    Napi::Value Chat(const Napi::CallbackInfo& info);
    Napi::Value Execute(const Napi::CallbackInfo& info);
    Napi::Value AddTool(const Napi::CallbackInfo& info);
    Napi::Value GetState(const Napi::CallbackInfo& info);
    
    LLMSpellAgent* agent_;
};

Napi::FunctionReference AgentWrapper::constructor;

Napi::Object AgentWrapper::Init(Napi::Env env, Napi::Object exports) {
    Napi::HandleScope scope(env);
    
    Napi::Function func = DefineClass(env, "Agent", {
        InstanceMethod("chat", &AgentWrapper::Chat),
        InstanceMethod("execute", &AgentWrapper::Execute),
        InstanceMethod("addTool", &AgentWrapper::AddTool),
        InstanceMethod("getState", &AgentWrapper::GetState),
    });
    
    constructor = Napi::Persistent(func);
    constructor.SuppressDestruct();
    
    exports.Set("Agent", func);
    return exports;
}

AgentWrapper::AgentWrapper(const Napi::CallbackInfo& info) 
    : Napi::ObjectWrap<AgentWrapper>(info) {
    
    Napi::Env env = info.Env();
    Napi::HandleScope scope(env);
    
    if (info.Length() < 1 || !info[0].IsObject()) {
        Napi::TypeError::New(env, "Agent configuration object expected")
            .ThrowAsJavaScriptException();
        return;
    }
    
    Napi::Object config = info[0].As<Napi::Object>();
    
    // Extract configuration
    std::string system_prompt = "";
    if (config.Has("systemPrompt")) {
        system_prompt = config.Get("systemPrompt").As<Napi::String>().Utf8Value();
    }
    
    std::string provider = "openai";
    if (config.Has("provider")) {
        provider = config.Get("provider").As<Napi::String>().Utf8Value();
    }
    
    // Create agent
    LLMSpellAgentConfig agent_config = {
        .system_prompt = system_prompt.c_str(),
        .provider = provider.c_str(),
        .tools = nullptr,
        .tool_count = 0
    };
    
    agent_ = llmspell_agent_create(&agent_config);
    if (!agent_) {
        Napi::Error::New(env, "Failed to create agent")
            .ThrowAsJavaScriptException();
    }
}

AgentWrapper::~AgentWrapper() {
    if (agent_) {
        llmspell_agent_destroy(agent_);
    }
}

Napi::Value AgentWrapper::Chat(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    if (info.Length() < 1 || !info[0].IsString()) {
        Napi::TypeError::New(env, "String message expected")
            .ThrowAsJavaScriptException();
        return env.Null();
    }
    
    std::string message = info[0].As<Napi::String>().Utf8Value();
    
    // Execute chat (potentially async)
    LLMSpellResult result = llmspell_agent_chat(agent_, message.c_str());
    
    if (result.error) {
        Napi::Error::New(env, result.error_message)
            .ThrowAsJavaScriptException();
        return env.Null();
    }
    
    return Napi::String::New(env, result.response);
}

// Async version using libuv worker threads
class ChatWorker : public Napi::AsyncWorker {
public:
    ChatWorker(Napi::Function& callback, LLMSpellAgent* agent, const std::string& message)
        : Napi::AsyncWorker(callback), agent_(agent), message_(message) {}
        
    ~ChatWorker() {}
    
    void Execute() override {
        result_ = llmspell_agent_chat(agent_, message_.c_str());
    }
    
    void OnOK() override {
        Napi::HandleScope scope(Env());
        
        if (result_.error) {
            Callback().Call({
                Napi::String::New(Env(), result_.error_message),
                Env().Null()
            });
        } else {
            Callback().Call({
                Env().Null(),
                Napi::String::New(Env(), result_.response)
            });
        }
    }
    
private:
    LLMSpellAgent* agent_;
    std::string message_;
    LLMSpellResult result_;
};

Napi::Value AgentWrapper::ChatAsync(const Napi::CallbackInfo& info) {
    Napi::Env env = info.Env();
    
    if (info.Length() < 2 || !info[0].IsString() || !info[1].IsFunction()) {
        Napi::TypeError::New(env, "String message and callback function expected")
            .ThrowAsJavaScriptException();
        return env.Null();
    }
    
    std::string message = info[0].As<Napi::String>().Utf8Value();
    Napi::Function callback = info[1].As<Napi::Function>();
    
    ChatWorker* worker = new ChatWorker(callback, agent_, message);
    worker->Queue();
    
    return env.Undefined();
}

// Module initialization
Napi::Object InitAll(Napi::Env env, Napi::Object exports) {
    // Initialize llmspell library
    if (!llmspell_init()) {
        Napi::Error::New(env, "Failed to initialize llmspell")
            .ThrowAsJavaScriptException();
        return exports;
    }
    
    AgentWrapper::Init(env, exports);
    ToolWrapper::Init(env, exports);
    WorkflowWrapper::Init(env, exports);
    
    // Add utility functions
    exports.Set("version", Napi::String::New(env, llmspell_version()));
    
    return exports;
}

NODE_API_MODULE(llmspell, InitAll)
```

### NPM Package Distribution

```json
{
  "name": "@rs/llmspell",
  "version": "1.0.0",
  "description": "Scriptable LLM interactions for Node.js",
  "main": "lib/index.js",
  "types": "lib/index.d.ts",
  "scripts": {
    "install": "node-gyp rebuild",
    "build": "node-gyp build",
    "test": "jest",
    "prebuild": "prebuildify --napi --strip"
  },
  "gypfile": true,
  "dependencies": {
    "node-addon-api": "^4.0.0"
  },
  "devDependencies": {
    "node-gyp": "^9.0.0",
    "prebuildify": "^5.0.0",
    "@types/node": "^18.0.0",
    "jest": "^29.0.0",
    "typescript": "^4.0.0"
  },
  "binary": {
    "module_name": "llmspell",
    "module_path": "./lib/binding/{platform}-{arch}",
    "remote_path": "https://github.com/company/rs-llmspell/releases/download/v{version}/",
    "package_name": "{module_name}-v{version}-{platform}-{arch}.tar.gz"
  },
  "keywords": ["ai", "llm", "agent", "automation", "rust"],
  "license": "MIT"
}
```

### TypeScript Definitions

```typescript
// index.d.ts - TypeScript definitions
export interface AgentConfig {
    systemPrompt?: string;
    provider?: string;
    model?: string;
    tools?: Tool[];
    maxTokens?: number;
    temperature?: number;
}

export interface Tool {
    name: string;
    description: string;
    schema: any;
    execute: (params: any) => Promise<any> | any;
}

export interface AgentExecutionResult {
    content: string;
    metadata: {
        tokens_used?: number;
        execution_time_ms?: number;
        tools_used?: string[];
    };
}

export declare class Agent {
    constructor(config: AgentConfig);
    
    chat(message: string): Promise<string>;
    chatSync(message: string): string;
    execute(input: any): Promise<AgentExecutionResult>;
    addTool(tool: Tool): void;
    getState(): any;
}

export interface WorkflowStep {
    name: string;
    agent?: Agent;
    tool?: Tool;
    input?: any;
    output?: string;
}

export interface WorkflowConfig {
    type: 'sequential' | 'parallel' | 'conditional';
    steps: WorkflowStep[];
    timeout?: number;
}

export declare class Workflow {
    constructor(config: WorkflowConfig);
    execute(input: any): Promise<any>;
    pause(): Promise<string>; // Returns checkpoint
    resume(checkpoint: string): Promise<void>;
}

export declare function createTool(config: {
    name: string;
    description: string;
    schema: any;
    execute: (params: any) => any;
}): Tool;

export declare function version(): string;
```

### JavaScript Module Usage Examples

```javascript
// ESM usage
import { Agent, createTool, Workflow } from '@rs/llmspell';

// Create tools
const calculatorTool = createTool({
    name: 'calculator',
    description: 'Perform mathematical calculations',
    schema: {
        type: 'object',
        properties: {
            expression: { type: 'string' }
        }
    },
    execute: (params) => {
        return eval(params.expression);
    }
});

// Create agent
const agent = new Agent({
    systemPrompt: 'You are a math tutor',
    provider: 'openai',
    model: 'gpt-4',
    tools: [calculatorTool]
});

// Use agent
const response = await agent.chat('What is the square root of 144?');
console.log('Agent:', response);

// Express.js integration
import express from 'express';
const app = express();

app.post('/chat', async (req, res) => {
    try {
        const { message } = req.body;
        const response = await agent.chat(message);
        res.json({ response });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// React component usage
import React, { useState } from 'react';
import { Agent } from '@rs/llmspell';

const ChatComponent = () => {
    const [agent] = useState(() => new Agent({
        systemPrompt: 'You are a helpful assistant',
        provider: 'anthropic'
    }));
    
    const [message, setMessage] = useState('');
    const [response, setResponse] = useState('');
    
    const handleChat = async () => {
        const result = await agent.chat(message);
        setResponse(result);
    };
    
    return (
        <div>
            <input 
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                placeholder="Ask a question..."
            />
            <button onClick={handleChat}>Send</button>
            <div>{response}</div>
        </div>
    );
};
```

## Architecture Changes for Module Support

### Core Library Restructuring

**Current Structure**:
```
rs-llmspell/
├── llmspell-core/          # Core traits and types
├── llmspell-agents/        # Agent implementations  
├── llmspell-tools/         # Tool implementations
├── llmspell-workflows/     # Workflow implementations
├── llmspell-bridge/        # Script engine bridges
└── llmspell-cli/          # CLI application
```

**Proposed Structure with Module Support**:
```
rs-llmspell/
├── llmspell-core/          # Core traits and types
├── llmspell-agents/        # Agent implementations
├── llmspell-tools/         # Tool implementations  
├── llmspell-workflows/     # Workflow implementations
├── llmspell-bridge/        # Script engine bridges (embedded)
│
├── llmspell-c-api/         # C API wrapper
├── llmspell-module-core/   # Module interface abstractions
│
├── bindings/
│   ├── lua/
│   │   ├── src/            # Lua C module source
│   │   ├── rockspec/       # LuaRocks specifications
│   │   └── examples/       # Usage examples
│   │
│   ├── node/
│   │   ├── src/            # N-API addon source
│   │   ├── package.json    # NPM package configuration
│   │   ├── binding.gyp     # Node-gyp build configuration
│   │   └── lib/            # TypeScript definitions
│   │
│   ├── python/             # Future: Python bindings
│   ├── ruby/               # Future: Ruby bindings
│   └── wasm/               # Future: WebAssembly module
│
├── llmspell-cli/          # CLI application
└── examples/
    ├── embedded/          # Embedded runtime examples
    └── module/            # Module usage examples
```

### C API Design

```c
// llmspell.h - Public C API
#ifndef LLMSPELL_H
#define LLMSPELL_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque types
typedef struct LLMSpellAgent LLMSpellAgent;
typedef struct LLMSpellTool LLMSpellTool;
typedef struct LLMSpellWorkflow LLMSpellWorkflow;

// Result type for operations
typedef struct {
    bool error;
    const char* error_message;
    const char* response;
    const char* metadata_json;
} LLMSpellResult;

// Agent configuration
typedef struct {
    const char* system_prompt;
    const char* provider;
    const char* model;
    LLMSpellTool** tools;
    size_t tool_count;
    const char* config_json;
} LLMSpellAgentConfig;

// Library initialization
bool llmspell_init(void);
void llmspell_cleanup(void);
const char* llmspell_version(void);

// Agent functions
LLMSpellAgent* llmspell_agent_create(const LLMSpellAgentConfig* config);
void llmspell_agent_destroy(LLMSpellAgent* agent);
LLMSpellResult llmspell_agent_chat(LLMSpellAgent* agent, const char* message);
LLMSpellResult llmspell_agent_execute(LLMSpellAgent* agent, const char* input_json);

// Tool functions
typedef LLMSpellResult (*LLMSpellToolCallback)(const char* params_json, void* user_data);

LLMSpellTool* llmspell_tool_create(
    const char* name,
    const char* description, 
    const char* schema_json,
    LLMSpellToolCallback callback,
    void* user_data
);
void llmspell_tool_destroy(LLMSpellTool* tool);

// Workflow functions
LLMSpellWorkflow* llmspell_workflow_create(const char* config_json);
void llmspell_workflow_destroy(LLMSpellWorkflow* workflow);
LLMSpellResult llmspell_workflow_execute(LLMSpellWorkflow* workflow, const char* input_json);

// Memory management
void llmspell_free_string(const char* str);

#ifdef __cplusplus
}
#endif

#endif // LLMSPELL_H
```

### Module Interface Abstractions

```rust
// llmspell-module-core/src/lib.rs
use llmspell_core::{Agent, Tool, Workflow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Common module interface
pub trait ModuleInterface: Send + Sync {
    fn init() -> Result<(), ModuleError>;
    fn create_agent(&self, config: AgentConfig) -> Result<Box<dyn Agent>, ModuleError>;
    fn create_tool(&self, config: ToolConfig) -> Result<Box<dyn Tool>, ModuleError>;
    fn create_workflow(&self, config: WorkflowConfig) -> Result<Box<dyn Workflow>, ModuleError>;
    fn version(&self) -> &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub system_prompt: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub tools: Vec<String>, // Tool names
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolConfig {
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value,
    pub implementation: ToolImplementation,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ToolImplementation {
    Native { function_name: String },
    Script { code: String, language: String },
    External { endpoint: String, auth: Option<String> },
}

// Error handling for modules
#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

// Module registry for managing different language bindings
pub struct ModuleRegistry {
    interfaces: HashMap<String, Box<dyn ModuleInterface>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            interfaces: HashMap::new(),
        }
    }
    
    pub fn register_interface(&mut self, name: String, interface: Box<dyn ModuleInterface>) {
        self.interfaces.insert(name, interface);
    }
    
    pub fn get_interface(&self, name: &str) -> Option<&dyn ModuleInterface> {
        self.interfaces.get(name).map(|i| i.as_ref())
    }
}
```

This comprehensive research provides the foundation for extending rs-llmspell beyond embedded scripting to full module support, enabling seamless integration with existing scripting environments and applications.