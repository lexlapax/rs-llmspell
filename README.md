# rs-llmspell

**Cast scripting spells to animate LLM golems** 🧙‍♂️✨

rs-llmspell makes LLM interactions scriptable in Rust. Write spells that control AI agents, tools, and workflows—all with Rust's safety, performance, and reliability.

```rust
// Example spell using rs-llmspell
use rs_llmspell::{SpellChecker, Agent};

let agent = Agent::new()
    .with_model("claude-3-opus")
    .with_system("You are a creative writer with vivid imagination.");

let story = agent.run("Write a short story about quantum computing")?;
println!("Story created: {}", story);
```

## 🚀 Key Features

- **🦀 Rust Performance**: Memory safety with zero-cost abstractions
- **🤖 Agent Orchestration**: AI agents with tools and workflows  
- **⚡ Native Speed**: Compiled performance without scripting overhead
- **🔒 Type Safety**: Compile-time guarantees for LLM interactions
- **🌉 Library Architecture**: Clean, composable API design

## 🏗️ Project Status

- 🚧 **Phase 1** - Core Library Foundation [IN PROGRESS]
  - ✅ Project structure and basic API design
  - 🔲 LLM provider integrations
  - 🔲 Agent system implementation
  - 🔲 Tool orchestration framework

## 🛠️ Quick Start

### Installation
Add to your `Cargo.toml`:
```toml
[dependencies]
rs-llmspell = "0.1.0"
```

### Basic Usage
```rust
use rs_llmspell::SpellChecker;

let checker = SpellChecker::new();
let result = checker.check_text("Your text here");
println!("{}", result);
```

## 🏛️ Architecture

```
┌─────────────────────────┐
│   Rust Applications     │  ← Your Rust code
├─────────────────────────┤
│   rs-llmspell API       │  ← Type-safe LLM interface
├─────────────────────────┤  
│   Provider Layer        │  ← LLM provider abstraction
├─────────────────────────┤
│   HTTP/API Clients      │  ← Network communication
└─────────────────────────┘
```

**Key Principle**: Provide a safe, fast, and ergonomic Rust interface to LLM services.

## 🔮 Planned Features

### Simple LLM Interaction
```rust
let response = llm::complete(CompleteRequest {
    model: "gpt-4".into(),
    prompt: "Explain quantum computing".into(),
})?;
println!("{}", response.content);
```

### Agent with Tools
```rust
let agent = Agent::new()
    .with_model("claude-3-opus")
    .with_tools(vec!["calculator", "web_search"]);
let result = agent.run("What's 15% of 2847?")?;
```

### Workflow Orchestration
```rust
let workflow = Workflow::sequential()
    .add_step("research", researcher_agent)
    .add_step("summarize", summarizer_agent)
    .add_step("save", file_writer_tool);
let result = workflow.run(WorkflowInput::new("climate change"))?;
```

## 🤝 Contributing

We welcome contributions! Please see our development guidelines:

- **TDD workflow**: Write tests first, then implement
- **Code standards**: Follow Rust best practices and clippy suggestions
- **Documentation**: All public APIs must have rustdoc comments

**Quick development workflow:**
```bash
git checkout -b feature/my-feature
cargo test  # Must pass before submitting
cargo clippy -- -D warnings
cargo fmt
```

## 📦 Core Dependencies

- [**tokio**](https://tokio.rs/) - Async runtime for LLM API calls
- [**serde**](https://serde.rs/) - Serialization for API communication
- [**reqwest**](https://docs.rs/reqwest/) - HTTP client for LLM providers

## 📄 License

MIT OR Apache-2.0 - see LICENSE files for details.

---

**⚡ Ready to cast your first spell?** Check out the examples in the `examples/` directory!
