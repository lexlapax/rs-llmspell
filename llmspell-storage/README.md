# llmspell-storage

State management and persistence for Rs-LLMSpell.

## Features
- Trait-based storage abstraction (sled for dev, rocksdb for production)
- Agent state preservation across handoffs
- Conversation history and checkpoint management

## Usage
```rust
use llmspell_storage::{StateStore, StorageBackend};

let store = StateStore::new(StorageBackend::Sled("./data"))?;
store.save_agent_state(agent_id, &state).await?;
let state = store.load_agent_state(agent_id).await?;
```

## Dependencies
- `llmspell-core` - State trait definitions
- External: `sled`, `rocksdb` (feature-gated)
- `llmspell-config` - Storage configuration