# llmspell-security

Security and sandboxing for Rs-LLMSpell.

## Features
- Script execution sandboxing with resource limits
- Tool permission system with capability-based access control
- Sensitive data redaction in logs and outputs

## Usage
```rust
use llmspell_security::{Sandbox, SecurityPolicy};

let policy = SecurityPolicy::default()
    .allow_network(false)
    .max_memory(100_000_000)  // 100MB
    .timeout(Duration::from_secs(30));
let sandbox = Sandbox::new(policy);
sandbox.execute(untrusted_code).await?;
```

## Dependencies
- `llmspell-core` - Security trait definitions
- `llmspell-hooks` - Security event monitoring
- Platform-specific sandboxing libraries