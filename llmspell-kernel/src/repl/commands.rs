//! REPL command definitions and parsing
//!
//! Defines all REPL meta-commands and debug commands.

use anyhow::Result;
use std::path::PathBuf;

/// REPL command types
#[derive(Debug, Clone)]
pub enum ReplCommand {
    /// Execute code
    Execute(String),
    /// Meta command
    Meta(MetaCommand),
    /// Debug command
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
            return MetaCommand::parse(stripped).map(ReplCommand::Meta);
        }

        // Check for debug commands (start with debug: or db:)
        if let Some(stripped) = trimmed.strip_prefix("debug:") {
            return DebugCommand::parse(stripped.trim()).map(ReplCommand::Debug);
        }
        if let Some(stripped) = trimmed.strip_prefix("db:") {
            return DebugCommand::parse(stripped.trim()).map(ReplCommand::Debug);
        }

        // Otherwise it's code to execute
        Ok(ReplCommand::Execute(trimmed.to_string()))
    }
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
