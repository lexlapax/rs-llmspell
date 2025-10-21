//! REPL command definitions and parsing
//!
//! Defines all REPL meta-commands and debug commands.

use anyhow::Result;
use std::path::PathBuf;

/// REPL command types
#[derive(Debug, Clone)]
pub enum ReplCommand {
    /// Execute code (Lua/JS)
    Execute(String),
    /// Chat with LLM agent
    Chat(String),
    /// Meta command (file/session operations)
    Meta(MetaCommand),
    /// Chat meta command (chat-specific operations)
    ChatMeta(ChatMetaCommand),
    /// Debug command (code debugging)
    Debug(DebugCommand),
    /// Empty line
    Empty,
}

impl ReplCommand {
    /// Parse command from input string
    ///
    /// # Errors
    ///
    /// Returns an error if the command cannot be parsed
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(ReplCommand::Empty);
        }

        // Check for meta commands (start with .)
        if let Some(stripped) = trimmed.strip_prefix('.') {
            // Try chat meta commands first
            if let Ok(chat_cmd) = ChatMetaCommand::parse(stripped) {
                return Ok(ReplCommand::ChatMeta(chat_cmd));
            }
            // Try explicit .chat command
            if let Some(message) = stripped.strip_prefix("chat ") {
                return Ok(ReplCommand::Chat(message.to_string()));
            }
            // Fall back to regular meta commands
            return MetaCommand::parse(stripped).map(ReplCommand::Meta);
        }

        // Check for debug commands (start with debug: or db:)
        if let Some(stripped) = trimmed.strip_prefix("debug:") {
            return DebugCommand::parse(stripped.trim()).map(ReplCommand::Debug);
        }
        if let Some(stripped) = trimmed.strip_prefix("db:") {
            return DebugCommand::parse(stripped.trim()).map(ReplCommand::Debug);
        }

        // Auto-detect: code or chat?
        match Self::detect_input_mode(trimmed) {
            InputMode::Code => Ok(ReplCommand::Execute(trimmed.to_string())),
            InputMode::Chat => Ok(ReplCommand::Chat(trimmed.to_string())),
            InputMode::Ambiguous => {
                // Default to chat for ambiguous input (safer for UX)
                // User can use explicit .chat or code keywords to override
                Ok(ReplCommand::Chat(trimmed.to_string()))
            }
        }
    }

    /// Detect if input is code or chat based on heuristics
    fn detect_input_mode(input: &str) -> InputMode {
        // Strong code indicators (keywords and symbols)
        const CODE_KEYWORDS: &[&str] = &[
            "function", "local", "let", "const", "var", "if", "for", "while", "return",
            "end", "then", "do", "async", "await", "class", "import", "export",
        ];

        const CODE_SYMBOLS: &[&str] = &["{", "}", "==", "!=", "||", "&&", "=>"];

        // Strong chat indicators (conversational phrases)
        const CHAT_PHRASES: &[&str] = &[
            "what is", "how do", "can you", "please", "explain", "tell me",
            "why", "when", "where", "who", "could you", "would you",
            "i need", "help me", "need help", "understanding",
        ];

        let lower = input.to_lowercase();

        // Check strong chat indicators first (these override code detection)
        // 1. Question mark = strong chat indicator
        if input.ends_with('?') {
            return InputMode::Chat;
        }

        // 2. Chat phrases (conversational patterns)
        for phrase in CHAT_PHRASES {
            if lower.contains(phrase) {
                return InputMode::Chat;
            }
        }

        // Check for code symbols (strong code indicators)
        for symbol in CODE_SYMBOLS {
            if input.contains(symbol) {
                return InputMode::Code;
            }
        }

        // Check for assignment operators (=, +=, -=, etc.)
        if input.contains(" = ") || input.contains("= ") && input.len() > 2 {
            return InputMode::Code;
        }

        // Check for code keywords
        for keyword in CODE_KEYWORDS {
            // Match as word boundary (not substring)
            if lower
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|word| word == *keyword)
            {
                return InputMode::Code;
            }
        }

        // Check word count: sentences with 5+ words without code symbols are likely chat
        let word_count = input.split_whitespace().count();
        if word_count >= 5 && !input.contains('{') && !input.contains(';') {
            return InputMode::Chat;
        }

        // Otherwise ambiguous
        InputMode::Ambiguous
    }
}

/// Input mode detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    /// Detected as code (Lua/JS)
    Code,
    /// Detected as chat (natural language)
    Chat,
    /// Ambiguous (could be either)
    Ambiguous,
}

/// REPL meta commands
#[derive(Debug, Clone)]
pub enum MetaCommand {
    /// Show help
    Help,
    /// Exit REPL
    Exit,
    /// Clear screen
    Clear,
    /// Save session
    Save(PathBuf),
    /// Load session
    Load(PathBuf),
    /// Show history
    History,
    /// Clear history
    ClearHistory,
    /// List variables
    Variables,
    /// Set variable
    Set(String, String),
    /// Unset variable
    Unset(String),
    /// Change directory
    Cd(PathBuf),
    /// Print working directory
    Pwd,
    /// List files
    Ls(Option<PathBuf>),
    /// Show system info
    Info,
    /// Reset session
    Reset,
    /// Run script file
    Run { file: PathBuf, args: Vec<String> },
    /// Toggle performance monitoring
    Perf { enabled: bool },
}

impl MetaCommand {
    /// Parse meta command from string (without leading .)
    ///
    /// # Errors
    ///
    /// Returns an error if the meta command cannot be parsed
    pub fn parse(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty meta command"));
        }

        match parts[0] {
            "help" | "h" | "?" => Ok(MetaCommand::Help),
            "exit" | "quit" | "q" => Ok(MetaCommand::Exit),
            "clear" | "cls" => Ok(MetaCommand::Clear),
            "save" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: .save <filename>"))
                } else {
                    Ok(MetaCommand::Save(PathBuf::from(parts[1])))
                }
            }
            "load" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: .load <filename>"))
                } else {
                    Ok(MetaCommand::Load(PathBuf::from(parts[1])))
                }
            }
            "history" | "hist" => Ok(MetaCommand::History),
            "clear-history" => Ok(MetaCommand::ClearHistory),
            "variables" | "vars" => Ok(MetaCommand::Variables),
            "set" => {
                if parts.len() < 3 {
                    Err(anyhow::anyhow!("Usage: .set <name> <value>"))
                } else {
                    let name = parts[1].to_string();
                    let value = parts[2..].join(" ");
                    Ok(MetaCommand::Set(name, value))
                }
            }
            "unset" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: .unset <name>"))
                } else {
                    Ok(MetaCommand::Unset(parts[1].to_string()))
                }
            }
            "cd" => {
                if parts.len() < 2 {
                    Ok(MetaCommand::Cd(dirs::home_dir().unwrap_or_default()))
                } else {
                    Ok(MetaCommand::Cd(PathBuf::from(parts[1])))
                }
            }
            "pwd" => Ok(MetaCommand::Pwd),
            "ls" => {
                if parts.len() < 2 {
                    Ok(MetaCommand::Ls(None))
                } else {
                    Ok(MetaCommand::Ls(Some(PathBuf::from(parts[1]))))
                }
            }
            "info" => Ok(MetaCommand::Info),
            "reset" => Ok(MetaCommand::Reset),
            "run" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: .run <script> [args...]\nExamples:\n  .run hello.lua\n  .run test.lua arg1 arg2"))
                } else {
                    let file = PathBuf::from(parts[1]);
                    let args = parts[2..].iter().map(|s| (*s).to_string()).collect();
                    Ok(MetaCommand::Run { file, args })
                }
            }
            "perf" => {
                let enabled = if parts.len() > 1 {
                    match parts[1] {
                        "on" | "true" | "1" => true,
                        "off" | "false" | "0" => false,
                        _ => return Err(anyhow::anyhow!("Usage: .perf [on|off]")),
                    }
                } else {
                    true // Toggle to on if no argument
                };
                Ok(MetaCommand::Perf { enabled })
            }
            _ => Err(anyhow::anyhow!("Unknown meta command: {}", parts[0])),
        }
    }

    /// Get help text for meta commands
    pub fn help_text() -> &'static str {
        r"REPL Meta Commands:
  .help, .h, .?        Show this help message
  .exit, .quit, .q     Exit the REPL
  .clear, .cls         Clear the screen
  .save <file>         Save session to file
  .load <file>         Load session from file
  .history, .hist      Show command history
  .clear-history       Clear command history
  .variables, .vars    List session variables
  .set <name> <value>  Set a variable
  .unset <name>        Unset a variable
  .cd <path>           Change directory
  .pwd                 Print working directory
  .ls [path]           List files
  .info                Show system info
  .reset               Reset session state
  .run <script> [args] Run a script file with optional arguments
  .perf [on|off]       Toggle performance monitoring"
    }
}

/// Chat meta commands for LLM interaction
#[derive(Debug, Clone)]
pub enum ChatMetaCommand {
    /// Update system prompt
    System(String),
    /// Switch LLM model
    Model(String),
    /// Configure allowed tools (comma-separated list)
    Tools(Vec<String>),
    /// Show conversation context and settings
    Context,
    /// Clear chat history (keep session and code variables)
    ClearChat,
}

impl ChatMetaCommand {
    /// Parse chat meta command from string (without leading .)
    ///
    /// # Errors
    ///
    /// Returns an error if the chat meta command cannot be parsed
    pub fn parse(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty chat meta command"));
        }

        match parts[0] {
            "system" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!(
                        "Usage: .system <prompt>\nExample: .system \"You are a helpful Rust expert\""
                    ))
                } else {
                    // Join all parts after "system" and strip quotes if present
                    let prompt = parts[1..].join(" ");
                    let prompt = prompt.trim_matches('"').to_string();
                    Ok(ChatMetaCommand::System(prompt))
                }
            }
            "model" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!(
                        "Usage: .model <model>\nExample: .model ollama/llama3.2:3b"
                    ))
                } else {
                    Ok(ChatMetaCommand::Model(parts[1].to_string()))
                }
            }
            "tools" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!(
                        "Usage: .tools <tool1,tool2,...>\nExample: .tools web-searcher,calculator"
                    ))
                } else {
                    // Join all parts after "tools" and split by comma to get tool list
                    let tools_str = parts[1..].join(" ");
                    let tools: Vec<String> = tools_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if tools.is_empty() {
                        Err(anyhow::anyhow!("No tools specified"))
                    } else {
                        Ok(ChatMetaCommand::Tools(tools))
                    }
                }
            }
            "context" => Ok(ChatMetaCommand::Context),
            "clearchat" => Ok(ChatMetaCommand::ClearChat),
            _ => Err(anyhow::anyhow!("Unknown chat meta command: {}", parts[0])),
        }
    }

    /// Get help text for chat meta commands
    pub fn help_text() -> &'static str {
        r"Chat Meta Commands:
  .system <prompt>         Update system prompt for the agent
  .model <model>           Switch LLM model (e.g., ollama/llama3.2:3b)
  .tools <tool1,tool2>     Configure allowed tools (comma-separated)
  .context                 Show conversation history and current settings
  .clearchat               Clear chat history (keeps code session)"
    }
}

/// Debug commands
#[derive(Debug, Clone)]
pub enum DebugCommand {
    /// Set breakpoint
    Break(BreakpointSpec),
    /// Remove breakpoint
    Delete(usize),
    /// List breakpoints
    List,
    /// Step into
    Step,
    /// Step over
    Next,
    /// Step out
    Finish,
    /// Continue execution
    Continue,
    /// Show local variables
    Locals,
    /// Show stack trace
    Backtrace,
    /// Select stack frame
    Frame(usize),
    /// Evaluate expression
    Print(String),
    /// Set watch expression
    Watch(String),
    /// Remove watch
    Unwatch(usize),
    /// Enable breakpoint
    Enable(usize),
    /// Disable breakpoint
    Disable(usize),
    /// Show current location
    Where,
    /// Pause execution
    Pause,
}

impl DebugCommand {
    /// Parse debug command from string
    ///
    /// # Errors
    ///
    /// Returns an error if the debug command cannot be parsed
    pub fn parse(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty debug command"));
        }

        match parts[0] {
            "break" | "b" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: break <line> [condition]"))
                } else {
                    let spec = BreakpointSpec::parse(&parts[1..])?;
                    Ok(DebugCommand::Break(spec))
                }
            }
            "delete" | "d" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: delete <id>"))
                } else {
                    let id = parts[1]
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Invalid breakpoint ID"))?;
                    Ok(DebugCommand::Delete(id))
                }
            }
            "list" | "l" => Ok(DebugCommand::List),
            "step" | "s" => Ok(DebugCommand::Step),
            "next" | "n" => Ok(DebugCommand::Next),
            "finish" | "f" => Ok(DebugCommand::Finish),
            "continue" | "c" => Ok(DebugCommand::Continue),
            "locals" => Ok(DebugCommand::Locals),
            "backtrace" | "bt" | "where" | "w" => Ok(DebugCommand::Backtrace),
            "frame" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: frame <number>"))
                } else {
                    let frame = parts[1]
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Invalid frame number"))?;
                    Ok(DebugCommand::Frame(frame))
                }
            }
            "print" | "p" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: print <expression>"))
                } else {
                    let expr = parts[1..].join(" ");
                    Ok(DebugCommand::Print(expr))
                }
            }
            "watch" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: watch <expression>"))
                } else {
                    let expr = parts[1..].join(" ");
                    Ok(DebugCommand::Watch(expr))
                }
            }
            "unwatch" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: unwatch <id>"))
                } else {
                    let id = parts[1]
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Invalid watch ID"))?;
                    Ok(DebugCommand::Unwatch(id))
                }
            }
            "enable" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: enable <id>"))
                } else {
                    let id = parts[1]
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Invalid breakpoint ID"))?;
                    Ok(DebugCommand::Enable(id))
                }
            }
            "disable" => {
                if parts.len() < 2 {
                    Err(anyhow::anyhow!("Usage: disable <id>"))
                } else {
                    let id = parts[1]
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Invalid breakpoint ID"))?;
                    Ok(DebugCommand::Disable(id))
                }
            }
            "pause" => Ok(DebugCommand::Pause),
            _ => Err(anyhow::anyhow!("Unknown debug command: {}", parts[0])),
        }
    }

    /// Get help text for debug commands
    pub fn help_text() -> &'static str {
        r"Debug Commands:
  break, b <line> [cond]  Set breakpoint at line with optional condition
  delete, d <id>          Delete breakpoint by ID
  list, l                 List all breakpoints
  step, s                 Step into next statement
  next, n                 Step over next statement
  finish, f               Step out of current function
  continue, c             Continue execution
  locals                  Show local variables
  backtrace, bt, where    Show stack trace
  frame <n>               Select stack frame
  print, p <expr>         Evaluate and print expression
  watch <expr>            Set watch expression
  unwatch <id>            Remove watch by ID
  enable <id>             Enable breakpoint
  disable <id>            Disable breakpoint
  pause                   Pause execution"
    }
}

/// Breakpoint specification
#[derive(Debug, Clone)]
pub struct BreakpointSpec {
    /// File (optional)
    pub file: Option<String>,
    /// Line number
    pub line: usize,
    /// Condition (optional)
    pub condition: Option<String>,
}

impl BreakpointSpec {
    /// Parse breakpoint specification
    ///
    /// # Errors
    ///
    /// Returns an error if the breakpoint specification is invalid
    fn parse(parts: &[&str]) -> Result<Self> {
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Missing line number"));
        }

        // Parse line number or file:line
        let first = parts[0];
        let (file, line) = if first.contains(':') {
            let parts: Vec<&str> = first.split(':').collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!("Invalid breakpoint format"));
            }
            let line = parts[1]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid line number"))?;
            (Some(parts[0].to_string()), line)
        } else {
            let line = first
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid line number"))?;
            (None, line)
        };

        // Parse condition if present
        let condition = if parts.len() > 1 {
            Some(parts[1..].join(" "))
        } else {
            None
        };

        Ok(BreakpointSpec {
            file,
            line,
            condition,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    // ChatMetaCommand parsing tests
    #[test]
    fn test_parse_system_command() {
        let result = ChatMetaCommand::parse("system You are a helpful assistant");
        assert!(result.is_ok());
        match result.unwrap() {
            ChatMetaCommand::System(prompt) => {
                assert_eq!(prompt, "You are a helpful assistant");
            }
            _ => panic!("Expected System variant"),
        }
    }

    #[test]
    fn test_parse_system_command_with_quotes() {
        let result = ChatMetaCommand::parse("system \"You are a Rust expert\"");
        assert!(result.is_ok());
        match result.unwrap() {
            ChatMetaCommand::System(prompt) => {
                assert_eq!(prompt, "You are a Rust expert");
            }
            _ => panic!("Expected System variant"),
        }
    }

    #[test]
    fn test_parse_model_command() {
        let result = ChatMetaCommand::parse("model ollama/llama3.2:3b");
        assert!(result.is_ok());
        match result.unwrap() {
            ChatMetaCommand::Model(model) => {
                assert_eq!(model, "ollama/llama3.2:3b");
            }
            _ => panic!("Expected Model variant"),
        }
    }

    #[test]
    fn test_parse_tools_command() {
        let result = ChatMetaCommand::parse("tools web-searcher,calculator");
        assert!(result.is_ok());
        match result.unwrap() {
            ChatMetaCommand::Tools(tools) => {
                assert_eq!(tools, vec!["web-searcher", "calculator"]);
            }
            _ => panic!("Expected Tools variant"),
        }
    }

    #[test]
    fn test_parse_tools_command_with_spaces() {
        let result = ChatMetaCommand::parse("tools web-searcher, calculator, data-analyzer");
        assert!(result.is_ok());
        match result.unwrap() {
            ChatMetaCommand::Tools(tools) => {
                assert_eq!(
                    tools,
                    vec!["web-searcher", "calculator", "data-analyzer"]
                );
            }
            _ => panic!("Expected Tools variant"),
        }
    }

    #[test]
    fn test_parse_context_command() {
        let result = ChatMetaCommand::parse("context");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ChatMetaCommand::Context));
    }

    #[test]
    fn test_parse_clearchat_command() {
        let result = ChatMetaCommand::parse("clearchat");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ChatMetaCommand::ClearChat));
    }

    #[test]
    fn test_parse_invalid_chat_meta_command() {
        let result = ChatMetaCommand::parse("invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown chat meta command"));
    }

    // ReplCommand parsing tests for chat
    #[test]
    fn test_parse_chat_meta_system() {
        let result = ReplCommand::parse(".system You are helpful");
        assert!(result.is_ok());
        match result.unwrap() {
            ReplCommand::ChatMeta(ChatMetaCommand::System(prompt)) => {
                assert_eq!(prompt, "You are helpful");
            }
            _ => panic!("Expected ChatMeta System variant"),
        }
    }

    #[test]
    fn test_parse_chat_meta_model() {
        let result = ReplCommand::parse(".model gpt-4");
        assert!(result.is_ok());
        match result.unwrap() {
            ReplCommand::ChatMeta(ChatMetaCommand::Model(model)) => {
                assert_eq!(model, "gpt-4");
            }
            _ => panic!("Expected ChatMeta Model variant"),
        }
    }

    #[test]
    fn test_parse_explicit_chat_command() {
        let result = ReplCommand::parse(".chat What is Rust?");
        assert!(result.is_ok());
        match result.unwrap() {
            ReplCommand::Chat(message) => {
                assert_eq!(message, "What is Rust?");
            }
            _ => panic!("Expected Chat variant"),
        }
    }

    // Input mode detection tests
    #[test]
    fn test_detect_code_function_keyword() {
        let mode = ReplCommand::detect_input_mode("function foo() return 42 end");
        assert_eq!(mode, InputMode::Code);
    }

    #[test]
    fn test_detect_code_local_keyword() {
        let mode = ReplCommand::detect_input_mode("local x = 5");
        assert_eq!(mode, InputMode::Code);
    }

    #[test]
    fn test_detect_code_if_keyword() {
        let mode = ReplCommand::detect_input_mode("if x > 10 then print(x) end");
        assert_eq!(mode, InputMode::Code);
    }

    #[test]
    fn test_detect_code_braces() {
        let mode = ReplCommand::detect_input_mode("{ key: value }");
        assert_eq!(mode, InputMode::Code);
    }

    #[test]
    fn test_detect_code_assignment() {
        let mode = ReplCommand::detect_input_mode("result = 2 + 2");
        assert_eq!(mode, InputMode::Code);
    }

    #[test]
    fn test_detect_chat_question_mark() {
        let mode = ReplCommand::detect_input_mode("What is Rust?");
        assert_eq!(mode, InputMode::Chat);
    }

    #[test]
    fn test_detect_chat_what_is_phrase() {
        let mode = ReplCommand::detect_input_mode("what is async programming");
        assert_eq!(mode, InputMode::Chat);
    }

    #[test]
    fn test_detect_chat_explain_phrase() {
        let mode = ReplCommand::detect_input_mode("explain ownership in Rust");
        assert_eq!(mode, InputMode::Chat);
    }

    #[test]
    fn test_detect_chat_can_you_phrase() {
        let mode = ReplCommand::detect_input_mode("can you help me with this problem");
        assert_eq!(mode, InputMode::Chat);
    }

    #[test]
    fn test_detect_chat_long_sentence() {
        let mode = ReplCommand::detect_input_mode("I need help understanding how async works");
        assert_eq!(mode, InputMode::Chat);
    }

    #[test]
    fn test_detect_ambiguous_short_input() {
        let mode = ReplCommand::detect_input_mode("hello");
        assert_eq!(mode, InputMode::Ambiguous);
    }

    #[test]
    fn test_parse_auto_detect_code() {
        let result = ReplCommand::parse("local x = 42");
        assert!(result.is_ok());
        match result.unwrap() {
            ReplCommand::Execute(code) => {
                assert_eq!(code, "local x = 42");
            }
            _ => panic!("Expected Execute variant for code"),
        }
    }

    #[test]
    fn test_parse_auto_detect_chat() {
        let result = ReplCommand::parse("What is the capital of France?");
        assert!(result.is_ok());
        match result.unwrap() {
            ReplCommand::Chat(message) => {
                assert_eq!(message, "What is the capital of France?");
            }
            _ => panic!("Expected Chat variant"),
        }
    }

    #[test]
    fn test_parse_auto_detect_ambiguous_defaults_to_chat() {
        let result = ReplCommand::parse("hello there");
        assert!(result.is_ok());
        match result.unwrap() {
            ReplCommand::Chat(message) => {
                assert_eq!(message, "hello there");
            }
            _ => panic!("Expected Chat variant for ambiguous input"),
        }
    }

    // Integration tests: ensure existing commands still work
    #[test]
    fn test_existing_meta_command_exit() {
        let result = ReplCommand::parse(".exit");
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            ReplCommand::Meta(MetaCommand::Exit)
        ));
    }

    #[test]
    fn test_existing_meta_command_help() {
        let result = ReplCommand::parse(".help");
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            ReplCommand::Meta(MetaCommand::Help)
        ));
    }

    #[test]
    fn test_existing_debug_command() {
        let result = ReplCommand::parse("debug: break 10");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ReplCommand::Debug(_)));
    }

    #[test]
    fn test_empty_input() {
        let result = ReplCommand::parse("");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ReplCommand::Empty));
    }
}
