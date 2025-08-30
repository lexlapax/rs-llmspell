# Phase 9: Interactive REPL and Debugging Infrastructure - TODO List

**Version**: 1.0  
**Date**: January 2025  
**Status**: Implementation Ready  
**Phase**: 9 (Interactive REPL and Debugging Infrastructure)  
**Timeline**: Weeks 30-32 (15 working days)  
**Priority**: HIGH (Developer Experience - Critical for adoption)  
**Dependencies**: Phase 8 Vector Storage ✅  
**Arch-Document**: docs/technical/master-architecture-vision.md  
**All-Phases-Document**: docs/in-progress/implementation-phases.md  
**Design-Document**: docs/in-progress/phase-09-design-doc.md (To be created)  
**Debug-Architecture**: docs/technical/operational-guide.md (debug material to be updated/created)  
**This-document**: working copy /TODO.md (pristine/immutable copy in docs/in-progress/PHASE09-DONE.md)

> **📋 Actionable Task List**: This document breaks down Phase 9 implementation into specific, measurable tasks for building a comprehensive REPL with integrated debugging capabilities that transforms the developer experience.

---

## Overview

**Goal**: Implement an interactive REPL with comprehensive debugging infrastructure, making llmspell development intuitive and productive. The REPL serves as the primary development interface with debugging naturally integrated, not as a separate mode.

**Success Criteria Summary:**
- [ ] REPL starts and accepts commands interactively
- [ ] State persists via Phase 5 state management integration
- [ ] Multi-line scripts with `<<<`/`>>>` markers work correctly
- [ ] Tab completion works for APIs and variables
- [ ] Command history is saved and restored
- [ ] Enhanced error messages show source context and suggestions
- [ ] Breakpoints can be set in files and source comments
- [ ] Step debugging works (step, next, continue, up/down)
- [ ] Variables inspected with lazy expansion
- [ ] Hot reload works both automatically and manually
- [ ] Script validation catches errors before execution
- [ ] Hook introspection commands functional
- [ ] Event stream monitored in real-time
- [ ] Full performance profiling with flamegraphs
- [ ] Both DAP and LSP protocols implemented
- [ ] Hook-based session recording with replay foundation
- [ ] All tests pass with >90% coverage
- [ ] Documentation complete with tutorials

---

## Phase 9.1: Core REPL Infrastructure (Days 1-3)

### Task 9.1.1: Create REPL Module Structure
**Priority**: CRITICAL  
**Estimated Time**: 3 hours  
**Assignee**: REPL Team Lead

**Description**: Set up the REPL module structure in llmspell-cli with proper dependencies.

**Acceptance Criteria:**
- [ ] `llmspell-cli/src/repl/` module created
- [ ] Dependencies added: `inquire`, custom loop components
- [ ] Basic module structure established
- [ ] Integration points with existing runtime identified
- [ ] `cargo check -p llmspell-cli` passes

**Implementation Steps:**
1. Create `llmspell-cli/src/repl/` directory
2. Add dependencies to `llmspell-cli/Cargo.toml`:
   ```toml
   inquire = "0.6"
   crossterm = "0.27"
   unicode-width = "0.1"
   syntect = "5.0"  # For syntax highlighting
   ```
3. Create module structure:
   ```rust
   pub mod core;      // REPL loop and state
   pub mod input;     // Input handling with inquire
   pub mod output;    // Output formatting and display
   pub mod commands;  // Meta-command processing
   pub mod completion; // Tab completion engine
   pub mod history;   // Command history management
   pub mod state;     // State persistence via Phase 5
   ```
4. Update `llmspell-cli/src/commands/repl.rs` to use new module
5. Verify compilation

**Definition of Done:**
- [ ] Module structure compiles without errors
- [ ] All submodules have basic structure
- [ ] Dependencies resolve correctly
- [ ] No clippy warnings

### Task 9.1.2: Implement Custom REPL Loop with Inquire
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: REPL Team

**Description**: Build the core REPL loop using inquire for prompts and custom evaluation.

**Acceptance Criteria:**
- [ ] Basic read-eval-print loop functional
- [ ] Inquire prompts for user input
- [ ] Script execution through existing runtime
- [ ] Graceful error handling
- [ ] Exit commands work (`.exit`, Ctrl+D)

**Implementation Steps:**
1. Create `ReplSession` struct in `core.rs`:
   ```rust
   pub struct ReplSession {
       runtime: Arc<ScriptRuntime>,
       state_manager: Arc<StateManager>, // From Phase 5
       prompt_number: usize,
       context: ReplContext,
   }
   ```
2. Implement main REPL loop:
   ```rust
   pub async fn run(&mut self) -> Result<()> {
       loop {
           let input = self.read_input().await?;
           match self.process_input(input).await {
               Ok(Continue) => continue,
               Ok(Exit) => break,
               Err(e) => self.display_error(e),
           }
       }
       Ok(())
   }
   ```
3. Use inquire for prompting:
   ```rust
   fn read_input(&self) -> Result<String> {
       inquire::Text::new(&format!("llmspell [{}]> ", self.prompt_number))
           .with_validator(|s: &str| Ok(()))
           .prompt()
   }
   ```
4. Handle meta-commands (`.help`, `.exit`, etc.)
5. Test basic interaction flow

**Definition of Done:**
- [ ] REPL loop starts and accepts input
- [ ] Commands execute and show output
- [ ] Errors don't crash the REPL
- [ ] Clean exit on `.exit` or Ctrl+D

### Task 9.1.3: Multi-line Input with <<< >>> Markers
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: REPL Team

**Description**: Implement explicit multi-line input mode with `<<<` and `>>>` markers.

**Acceptance Criteria:**
- [ ] `<<<` starts multi-line mode
- [ ] `>>>` ends multi-line mode and executes
- [ ] Line continuation shows appropriate prompt
- [ ] Nested markers handled correctly
- [ ] Syntax highlighting in multi-line mode

**Implementation Steps:**
1. Create `MultiLineBuffer` in `input.rs`:
   ```rust
   pub struct MultiLineBuffer {
       lines: Vec<String>,
       in_multiline: bool,
       depth: usize,
   }
   ```
2. Detect multi-line markers:
   ```rust
   fn detect_multiline_start(input: &str) -> bool {
       input.trim() == "<<<"
   }
   fn detect_multiline_end(input: &str) -> bool {
       input.trim() == ">>>"
   }
   ```
3. Implement multi-line collection:
   ```rust
   async fn collect_multiline(&mut self) -> Result<String> {
       let mut buffer = MultiLineBuffer::new();
       loop {
           let line = self.read_continuation_line()?;
           if detect_multiline_end(&line) {
               break;
           }
           buffer.add_line(line);
       }
       Ok(buffer.to_script())
   }
   ```
4. Add continuation prompt:
   ```rust
   fn read_continuation_line(&self) -> Result<String> {
       inquire::Text::new("... ")
           .prompt()
   }
   ```
5. Test with complex multi-line scripts

**Definition of Done:**
- [ ] Multi-line mode triggers on `<<<`
- [ ] Script executes on `>>>`
- [ ] Continuation prompts display correctly
- [ ] Multi-line scripts execute properly

### Task 9.1.4: Command History Management
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: REPL Team

**Description**: Implement command history with persistence and search capabilities.

**Acceptance Criteria:**
- [ ] Command history saved to file
- [ ] History persists between sessions
- [ ] Arrow keys navigate history
- [ ] History search with Ctrl+R pattern
- [ ] History limit configurable

**Implementation Steps:**
1. Create `HistoryManager` in `history.rs`:
   ```rust
   pub struct HistoryManager {
       history: VecDeque<HistoryEntry>,
       max_entries: usize,
       history_file: PathBuf,
   }
   
   pub struct HistoryEntry {
       command: String,
       timestamp: DateTime<Utc>,
       execution_time: Duration,
       success: bool,
   }
   ```
2. Implement persistence:
   ```rust
   impl HistoryManager {
       pub fn load_from_file(&mut self) -> Result<()> {
           // Load from ~/.llmspell_history
       }
       
       pub fn save_to_file(&self) -> Result<()> {
           // Save with rotation if > max_entries
       }
   }
   ```
3. Integrate with inquire for navigation:
   ```rust
   fn setup_history_navigation(&mut self, prompt: &mut Text) {
       prompt.with_history(self.history.iter().map(|e| &e.command));
   }
   ```
4. Add history search functionality:
   ```rust
   pub fn search_history(&self, pattern: &str) -> Vec<&HistoryEntry> {
       self.history.iter()
           .filter(|e| e.command.contains(pattern))
           .collect()
   }
   ```
5. Test history persistence and navigation

**Definition of Done:**
- [ ] History saves and loads correctly
- [ ] Arrow keys navigate through history
- [ ] Search functionality works
- [ ] History file properly managed

### Task 9.1.5: Tab Completion Engine
**Priority**: HIGH  
**Estimated Time**: 5 hours  
**Assignee**: REPL Team

**Description**: Build comprehensive tab completion for APIs, variables, and file paths.

**Acceptance Criteria:**
- [ ] Tab completes API names (Agent, Tool, Workflow, Debug)
- [ ] Tab completes variable names in scope
- [ ] Tab completes file paths
- [ ] Tab completes meta-commands
- [ ] Fuzzy matching for partial inputs

**Implementation Steps:**
1. Create `CompletionEngine` in `completion.rs`:
   ```rust
   pub struct CompletionEngine {
       api_completions: Vec<CompletionItem>,
       variable_tracker: VariableTracker,
       command_completions: Vec<CompletionItem>,
   }
   
   pub struct CompletionItem {
       pub text: String,
       pub kind: CompletionKind,
       pub documentation: Option<String>,
   }
   ```
2. Build API completion database:
   ```rust
   fn build_api_completions() -> Vec<CompletionItem> {
       vec![
           CompletionItem::new("Agent", Kind::Class),
           CompletionItem::new("Agent.builder()", Kind::Method),
           CompletionItem::new("Tool.invoke()", Kind::Method),
           // ... all APIs
       ]
   }
   ```
3. Track variables in scope:
   ```rust
   impl VariableTracker {
       pub fn track_assignment(&mut self, name: &str, type_hint: Option<&str>) {
           self.variables.insert(name.to_string(), VariableInfo { ... });
       }
       
       pub fn get_completions(&self, prefix: &str) -> Vec<CompletionItem> {
           self.variables.iter()
               .filter(|(name, _)| name.starts_with(prefix))
               .map(|(name, info)| CompletionItem::variable(name, info))
               .collect()
       }
   }
   ```
4. Integrate with inquire autocomplete:
   ```rust
   fn setup_completion(&self, prompt: &mut Text) {
       let engine = self.completion_engine.clone();
       prompt.with_autocomplete(move |input: &str| {
           engine.get_completions(input)
       });
   }
   ```
5. Add fuzzy matching:
   ```rust
   use fuzzy_matcher::FuzzyMatcher;
   
   fn fuzzy_complete(&self, input: &str) -> Vec<CompletionItem> {
       let matcher = SkimMatcherV2::default();
       self.all_completions.iter()
           .filter_map(|item| {
               matcher.fuzzy_match(&item.text, input)
                   .map(|score| (item, score))
           })
           .sorted_by_key(|(_, score)| -score)
           .map(|(item, _)| item.clone())
           .collect()
   }
   ```

**Definition of Done:**
- [ ] Tab completion triggers correctly
- [ ] API completions work
- [ ] Variable completions track scope
- [ ] File path completion functional
- [ ] Fuzzy matching provides good results

### Task 9.1.6: State Integration with Phase 5
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: REPL Team

**Description**: Integrate REPL state management with Phase 5's StateManager.

**Acceptance Criteria:**
- [ ] REPL state persists through StateManager
- [ ] Variables and context saved between sessions
- [ ] State can be explicitly saved/loaded
- [ ] State scoped to session/user/global
- [ ] Migration from old state formats

**Implementation Steps:**
1. Create `ReplStateAdapter` in `state.rs`:
   ```rust
   pub struct ReplStateAdapter {
       state_manager: Arc<StateManager>, // From Phase 5
       session_id: Uuid,
       scope: StateScope,
   }
   ```
2. Define REPL state structure:
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct ReplState {
       variables: HashMap<String, Value>,
       history: Vec<String>,
       breakpoints: Vec<Breakpoint>,
       watches: Vec<WatchExpression>,
       settings: ReplSettings,
   }
   ```
3. Implement state operations:
   ```rust
   impl ReplStateAdapter {
       pub async fn save_state(&self, state: &ReplState) -> Result<()> {
           self.state_manager.store(
               &self.session_id,
               "repl_state",
               state,
               self.scope.clone(),
           ).await
       }
       
       pub async fn load_state(&self) -> Result<Option<ReplState>> {
           self.state_manager.retrieve(
               &self.session_id,
               "repl_state",
               self.scope.clone(),
           ).await
       }
   }
   ```
4. Auto-save on changes:
   ```rust
   impl ReplSession {
       async fn after_execution(&mut self, result: &ExecutionResult) -> Result<()> {
           if result.modified_state {
               self.state_adapter.save_state(&self.current_state()).await?;
           }
           Ok(())
       }
   }
   ```
5. Test state persistence across sessions

**Definition of Done:**
- [ ] State saves automatically
- [ ] State loads on REPL start
- [ ] Variables persist between sessions
- [ ] State scoping works correctly

### Task 9.1.7: Section 9.1 Testing and Code Quality
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: QA Team

**Description**: Comprehensive testing and code quality validation for Core REPL Infrastructure.

**Acceptance Criteria:**
- [ ] All unit tests pass with 100% coverage for new code
- [ ] Integration tests validate REPL flow
- [ ] Zero clippy warnings with `cargo clippy --all-features`
- [ ] Code formatted with `cargo fmt --all`
- [ ] Documentation complete for all public APIs
- [ ] Performance benchmarks established

**Implementation Steps:**
1. Write unit tests for each REPL component:
   ```rust
   #[cfg(test)]
   mod repl_tests {
       use super::*;
       
       #[test]
       fn test_repl_session_creation() {
           let session = ReplSession::new(runtime, state_manager);
           assert!(session.is_ok());
       }
       
       #[test]
       fn test_multiline_input_detection() {
           assert!(detect_multiline_start("<<<"));
           assert!(detect_multiline_end(">>>"));
       }
       
       #[test]
       fn test_command_history_persistence() {
           let mut history = HistoryManager::new();
           history.add_entry("test command");
           history.save_to_file().unwrap();
           
           let mut loaded = HistoryManager::new();
           loaded.load_from_file().unwrap();
           assert_eq!(loaded.get_last(), Some("test command"));
       }
       
       #[test]
       fn test_tab_completion_api_names() {
           let engine = CompletionEngine::new();
           let completions = engine.get_completions("Age");
           assert!(completions.iter().any(|c| c.text == "Agent"));
       }
       
       #[test]
       fn test_state_integration() {
           let adapter = ReplStateAdapter::new(state_manager);
           let state = ReplState::default();
           adapter.save_state(&state).await.unwrap();
           let loaded = adapter.load_state().await.unwrap();
           assert_eq!(state, loaded);
       }
   }
   ```
2. Write integration tests:
   ```rust
   #[tokio::test]
   async fn test_repl_full_session() {
       let mut repl = create_test_repl().await;
       
       // Test simple command
       repl.process_input("print('hello')").await.unwrap();
       
       // Test multi-line
       repl.process_input("<<<").await.unwrap();
       repl.process_input("function test()").await.unwrap();
       repl.process_input("  return 42").await.unwrap();
       repl.process_input("end").await.unwrap();
       repl.process_input(">>>").await.unwrap();
       
       // Test meta-command
       repl.process_input(".exit").await.unwrap();
   }
   ```
3. Fix all clippy warnings:
   ```bash
   cargo clippy --all-features --all-targets -- -D warnings
   cargo clippy --all-features -- -W clippy::pedantic
   cargo clippy --all-features -- -W clippy::nursery
   ```
4. Format all code:
   ```bash
   cargo fmt --all -- --check
   cargo fmt --all
   ```
5. Run benchmarks:
   ```rust
   #[bench]
   fn bench_tab_completion(b: &mut Bencher) {
       let engine = CompletionEngine::new();
       b.iter(|| {
           engine.get_completions("Tool.inv")
       });
   }
   ```
6. Validate documentation:
   ```bash
   cargo doc --no-deps --all-features
   cargo test --doc
   ```

**Definition of Done:**
- [ ] `cargo test -p llmspell-cli -- repl::` passes
- [ ] `cargo clippy --all-features` shows zero warnings
- [ ] `cargo fmt --all -- --check` passes
- [ ] Code coverage >95% for new code
- [ ] Benchmarks run without regression
- [ ] Documentation builds without warnings

---

## Phase 9.2: Enhanced Error Reporting (Days 4-6)

### Task 9.2.1: Error Context Enhancement System
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: Debug Team Lead

**Description**: Build system to enhance Lua errors with source context and helpful information.

**Acceptance Criteria:**
- [ ] Errors show source file and line number
- [ ] Context shows surrounding code
- [ ] Local variables captured at error point
- [ ] Stack trace enhanced with details
- [ ] Suggestions provided for common errors

**Implementation Steps:**
1. Create `llmspell-bridge/src/lua/error_enhancer.rs`:
   ```rust
   pub struct EnhancedLuaError {
       pub original_error: String,
       pub script_path: Option<PathBuf>,
       pub location: ErrorLocation,
       pub source_context: SourceContext,
       pub stack_trace: EnhancedStackTrace,
       pub local_variables: HashMap<String, String>,
       pub suggestions: Vec<String>,
   }
   
   pub struct ErrorLocation {
       pub line: usize,
       pub column: Option<usize>,
       pub function_name: Option<String>,
   }
   
   pub struct SourceContext {
       pub lines_before: Vec<(usize, String)>,
       pub error_line: (usize, String),
       pub lines_after: Vec<(usize, String)>,
       pub highlight_range: Option<(usize, usize)>,
   }
   ```
2. Parse Lua error messages:
   ```rust
   impl EnhancedLuaError {
       pub fn from_lua_error(err: &mlua::Error, lua: &Lua) -> Self {
           let original = err.to_string();
           let location = Self::parse_error_location(&original);
           let source_context = Self::extract_source_context(&location);
           let stack_trace = Self::capture_enhanced_stack(lua);
           let locals = Self::capture_locals(lua, &location);
           let suggestions = Self::generate_suggestions(&original, &source_context);
           
           Self {
               original_error: original,
               location,
               source_context,
               stack_trace,
               local_variables: locals,
               suggestions,
           }
       }
   }
   ```
3. Extract source context:
   ```rust
   fn extract_source_context(location: &ErrorLocation) -> SourceContext {
       if let Some(path) = &location.script_path {
           let source = std::fs::read_to_string(path).ok()?;
           let lines: Vec<_> = source.lines().enumerate().collect();
           
           let start = location.line.saturating_sub(3);
           let end = (location.line + 3).min(lines.len());
           
           SourceContext {
               lines_before: lines[start..location.line-1].to_vec(),
               error_line: lines[location.line-1].clone(),
               lines_after: lines[location.line..end].to_vec(),
               highlight_range: Self::find_error_span(&lines[location.line-1].1),
           }
       } else {
           SourceContext::empty()
       }
   }
   ```
4. Capture local variables:
   ```rust
   fn capture_locals(lua: &Lua, location: &ErrorLocation) -> HashMap<String, String> {
       let mut locals = HashMap::new();
       
       // Use Lua debug API to get locals
       if let Ok(debug) = lua.globals().get::<_, Table>("debug") {
           // getlocal() at error frame
           for i in 1.. {
               match debug.call_function::<_, (Option<String>, Value)>(
                   "getlocal", (location.stack_level, i)
               ) {
                   Ok((Some(name), value)) if !name.starts_with('(') => {
                       locals.insert(name, format_value(&value));
                   }
                   _ => break,
               }
           }
       }
       
       locals
   }
   ```
5. Generate smart suggestions:
   ```rust
   fn generate_suggestions(error: &str, context: &SourceContext) -> Vec<String> {
       let mut suggestions = Vec::new();
       
       if error.contains("attempt to index a nil value") {
           suggestions.push("The variable might not be initialized. Check if it exists.".into());
           suggestions.push("Use 'if variable then ... end' to check before accessing.".into());
           
           // Find variable name from context
           if let Some(var_name) = Self::extract_nil_variable(&context.error_line.1) {
               // Look for similar names
               let similar = Self::find_similar_variables(var_name, context);
               if !similar.is_empty() {
                   suggestions.push(format!("Did you mean: {}?", similar.join(", ")));
               }
           }
       }
       
       if error.contains("attempt to call a nil value") {
           suggestions.push("The function doesn't exist. Check spelling and availability.".into());
       }
       
       if error.contains("bad argument") {
           suggestions.push("Check the function documentation for correct parameter types.".into());
           suggestions.push("Use Debug.dump() to inspect the value you're passing.".into());
       }
       
       suggestions
   }
   ```

**Definition of Done:**
- [ ] Error enhancement captures all context
- [ ] Source location extracted correctly
- [ ] Local variables captured
- [ ] Suggestions generated for common errors
- [ ] Performance impact minimal

### Task 9.2.2: Terminal-Aware Error Formatter
**Priority**: HIGH  
**Estimated Time**: 5 hours  
**Assignee**: Debug Team

**Description**: Create beautiful error display with configurable color support.

**Acceptance Criteria:**
- [ ] Errors display with colors when terminal supports it
- [ ] Fallback to plain text when no color support
- [ ] Configurable color schemes
- [ ] Source code syntax highlighted
- [ ] Error pointer shows exact location

**Implementation Steps:**
1. Create `error_formatter.rs`:
   ```rust
   use crossterm::style::{Color, Stylize};
   
   pub struct ErrorFormatter {
       color_mode: ColorMode,
       syntax_highlighter: SyntaxHighlighter,
   }
   
   pub enum ColorMode {
       Always,
       Never,
       Auto,
   }
   ```
2. Detect terminal capabilities:
   ```rust
   impl ErrorFormatter {
       pub fn new() -> Self {
           let color_mode = if std::env::var("NO_COLOR").is_ok() {
               ColorMode::Never
           } else if atty::is(atty::Stream::Stdout) {
               ColorMode::Auto
           } else {
               ColorMode::Never
           };
           
           Self {
               color_mode,
               syntax_highlighter: SyntaxHighlighter::new("lua"),
           }
       }
   }
   ```
3. Format enhanced errors:
   ```rust
   impl ErrorFormatter {
       pub fn format_error(&self, error: &EnhancedLuaError) -> String {
           let mut output = String::new();
           
           // Error header
           writeln!(output, "{} {}", 
               self.style("error:", Color::Red, true),
               error.original_error
           );
           
           // Location
           if let Some(path) = &error.script_path {
               writeln!(output, "  {} {}:{}:{}",
                   self.style("-->", Color::Blue, true),
                   path.display(),
                   error.location.line,
                   error.location.column.unwrap_or(1)
               );
           }
           
           // Source context with line numbers
           writeln!(output, "   {}", self.style("|", Color::Blue, false));
           
           for (line_no, line) in &error.source_context.lines_before {
               writeln!(output, "{:4} {} {}",
                   line_no,
                   self.style("|", Color::Blue, false),
                   self.dim(line)
               );
           }
           
           // Error line with highlighting
           let (line_no, line) = &error.source_context.error_line;
           writeln!(output, "{} {} {}",
               self.style(&format!("{:4}", line_no), Color::Red, true),
               self.style("|", Color::Red, false),
               self.syntax_highlight(line)
           );
           
           // Error pointer
           if let Some((start, end)) = error.source_context.highlight_range {
               writeln!(output, "     {} {}{}",
                   self.style("|", Color::Red, false),
                   " ".repeat(start),
                   self.style(&"^".repeat(end - start), Color::Red, true)
               );
           }
           
           // After context
           for (line_no, line) in &error.source_context.lines_after {
               writeln!(output, "{:4} {} {}",
                   line_no,
                   self.style("|", Color::Blue, false),
                   self.dim(line)
               );
           }
           
           // Suggestions
           if !error.suggestions.is_empty() {
               writeln!(output, "\n{} {}",
                   self.style("help:", Color::Green, true),
                   error.suggestions[0]
               );
               for suggestion in &error.suggestions[1..] {
                   writeln!(output, "      {}", suggestion);
               }
           }
           
           // Local variables
           if !error.local_variables.is_empty() {
               writeln!(output, "\n{} at error point:",
                   self.style("locals", Color::Yellow, true)
               );
               for (name, value) in &error.local_variables {
                   writeln!(output, "  {} = {}", 
                       self.style(name, Color::Cyan, false),
                       value
                   );
               }
           }
           
           output
       }
   }
   ```
4. Implement syntax highlighting:
   ```rust
   impl SyntaxHighlighter {
       pub fn highlight(&self, code: &str) -> String {
           if !self.enabled {
               return code.to_string();
           }
           
           // Use syntect for Lua syntax highlighting
           let syntax_set = SyntaxSet::load_defaults_newlines();
           let theme_set = ThemeSet::load_defaults();
           
           let syntax = syntax_set.find_syntax_by_extension("lua").unwrap();
           let mut highlighter = HighlightLines::new(syntax, &theme_set.themes["base16-ocean.dark"]);
           
           let ranges = highlighter.highlight_line(code, &syntax_set).unwrap();
           
           // Convert to terminal colors
           self.ranges_to_terminal_string(ranges)
       }
   }
   ```
5. Test with various terminal configurations

**Definition of Done:**
- [ ] Colors display correctly when supported
- [ ] Plain text fallback works
- [ ] Syntax highlighting functional
- [ ] Error formatting beautiful and readable
- [ ] Configuration options work

### Task 9.2.3: Common Error Pattern Detection
**Priority**: MEDIUM  
**Estimated Time**: 4 hours  
**Assignee**: Debug Team

**Description**: Build pattern matching for common Lua errors with specific suggestions.

**Acceptance Criteria:**
- [ ] Nil value errors detected and explained
- [ ] Type mismatch errors clarified
- [ ] Missing function/method errors handled
- [ ] Table access errors explained
- [ ] Async/tool call errors enhanced

**Implementation Steps:**
1. Create error pattern registry:
   ```rust
   pub struct ErrorPatternRegistry {
       patterns: Vec<ErrorPattern>,
   }
   
   pub struct ErrorPattern {
       regex: Regex,
       extractor: Box<dyn Fn(&Captures) -> ErrorInfo>,
       suggestion_generator: Box<dyn Fn(&ErrorInfo, &SourceContext) -> Vec<String>>,
   }
   ```
2. Register common patterns:
   ```rust
   impl ErrorPatternRegistry {
       pub fn new() -> Self {
           let mut registry = Self { patterns: vec![] };
           
           // Nil index error
           registry.register(
               r"attempt to index a nil value \(?(global|local|field) '([^']+)'",
               |caps| ErrorInfo::NilIndex {
                   scope: caps[1].to_string(),
                   name: caps[2].to_string(),
               },
               |info, ctx| {
                   let mut suggestions = vec![
                       format!("Variable '{}' is nil. Initialize it first.", info.name),
                       format!("Check with: if {} then ... end", info.name),
                   ];
                   
                   // Find similar names
                   if let Some(similar) = find_similar_in_context(&info.name, ctx) {
                       suggestions.push(format!("Did you mean '{}'?", similar));
                   }
                   
                   suggestions
               }
           );
           
           // Function not found
           registry.register(
               r"attempt to call a nil value \(?(method|field|global) '([^']+)'",
               |caps| ErrorInfo::NilCall {
                   kind: caps[1].to_string(),
                   name: caps[2].to_string(),
               },
               |info, _| vec![
                   format!("Function '{}' doesn't exist.", info.name),
                   "Check available methods with Debug.dump(object)".into(),
                   "Verify the API documentation for correct usage.".into(),
               ]
           );
           
           // Type mismatch
           registry.register(
               r"bad argument #(\d+) to '([^']+)' \(([^)]+) expected, got ([^)]+)\)",
               |caps| ErrorInfo::TypeMismatch {
                   arg_num: caps[1].parse().unwrap(),
                   function: caps[2].to_string(),
                   expected: caps[3].to_string(),
                   got: caps[4].to_string(),
               },
               |info, _| vec![
                   format!("Function '{}' expects {} for argument {}, but got {}",
                       info.function, info.expected, info.arg_num, info.got
                   ),
                   "Check the value with Debug.dump() before passing.".into(),
                   format!("Convert with to{}() if needed.", info.expected),
               ]
           );
           
           registry
       }
   }
   ```
3. Match and enhance errors:
   ```rust
   impl ErrorPatternRegistry {
       pub fn enhance_error(&self, error_msg: &str, context: &SourceContext) -> Vec<String> {
           for pattern in &self.patterns {
               if let Some(caps) = pattern.regex.captures(error_msg) {
                   let info = (pattern.extractor)(&caps);
                   return (pattern.suggestion_generator)(&info, context);
               }
           }
           
           // Default suggestions
           vec!["Check the Lua reference manual for more information.".into()]
       }
   }
   ```
4. Tool/Agent specific patterns:
   ```rust
   // Tool invocation errors
   registry.register(
       r"Tool '([^']+)' failed: (.+)",
       |caps| ErrorInfo::ToolError {
           tool: caps[1].to_string(),
           reason: caps[2].to_string(),
       },
       |info, _| vec![
           format!("Tool '{}' execution failed: {}", info.tool, info.reason),
           "Check tool parameters with Tool.schema(name)".into(),
           "Verify tool is registered with Tool.list()".into(),
       ]
   );
   
   // Agent errors
   registry.register(
       r"Agent '([^']+)' error: (.+)",
       |caps| ErrorInfo::AgentError {
           agent: caps[1].to_string(),
           error: caps[2].to_string(),
       },
       |info, _| vec![
           format!("Agent '{}' encountered an error: {}", info.agent, info.error),
           "Check agent configuration and model availability.".into(),
           "Use Debug.trace() to see agent execution flow.".into(),
       ]
   );
   ```
5. Test with real error scenarios

**Definition of Done:**
- [ ] Common error patterns detected
- [ ] Helpful suggestions generated
- [ ] Tool/Agent errors enhanced
- [ ] Pattern matching performant
- [ ] Extensible for new patterns

### Task 9.2.4: Section 9.2 Testing and Code Quality
**Priority**: CRITICAL  
**Estimated Time**: 3 hours  
**Assignee**: QA Team

**Description**: Comprehensive testing and code quality validation for Enhanced Error Reporting.

**Acceptance Criteria:**
- [ ] All error enhancement tests pass
- [ ] Error formatter tests cover all terminal modes
- [ ] Zero clippy warnings in error handling code
- [ ] Code formatted consistently
- [ ] Error patterns documented
- [ ] Performance impact <10ms

**Implementation Steps:**
1. Write unit tests for error enhancement:
   ```rust
   #[cfg(test)]
   mod error_tests {
       use super::*;
       
       #[test]
       fn test_error_context_extraction() {
           let lua_error = create_test_error("attempt to index nil");
           let enhanced = EnhancedLuaError::from_lua_error(&lua_error, &lua);
           
           assert!(enhanced.suggestions.len() > 0);
           assert!(enhanced.source_context.error_line.1.contains("nil"));
           assert!(!enhanced.local_variables.is_empty());
       }
       
       #[test]
       fn test_terminal_color_detection() {
           std::env::set_var("NO_COLOR", "1");
           let formatter = ErrorFormatter::new();
           assert_eq!(formatter.color_mode, ColorMode::Never);
           
           std::env::remove_var("NO_COLOR");
           let formatter = ErrorFormatter::new();
           assert_eq!(formatter.color_mode, ColorMode::Auto);
       }
       
       #[test]
       fn test_error_pattern_matching() {
           let registry = ErrorPatternRegistry::new();
           
           let suggestions = registry.enhance_error(
               "attempt to call a nil value (field 'foo')",
               &SourceContext::empty()
           );
           
           assert!(suggestions.iter().any(|s| s.contains("doesn't exist")));
       }
       
       #[test]
       fn test_suggestion_generation() {
           let error = "bad argument #2 to 'Tool.invoke'";
           let suggestions = generate_suggestions(error, &context);
           
           assert!(suggestions.iter().any(|s| s.contains("Check the value")));
       }
   }
   ```
2. Write integration tests for error flow:
   ```rust
   #[tokio::test]
   async fn test_error_enhancement_e2e() {
       let runtime = create_test_runtime();
       
       // Trigger various errors
       let result = runtime.execute_script("local x = nil; x.foo").await;
       assert!(result.is_err());
       
       let enhanced = enhance_error(result.unwrap_err());
       assert!(enhanced.suggestions.len() > 0);
       
       // Test formatting
       let formatted = ErrorFormatter::new().format_error(&enhanced);
       assert!(formatted.contains("error:"));
       assert!(formatted.contains("-->"));
   }
   ```
3. Performance benchmarks:
   ```rust
   #[bench]
   fn bench_error_enhancement(b: &mut Bencher) {
       let error = create_complex_error();
       b.iter(|| {
           EnhancedLuaError::from_lua_error(&error, &lua)
       });
   }
   
   #[bench]
   fn bench_error_formatting(b: &mut Bencher) {
       let enhanced = create_enhanced_error();
       let formatter = ErrorFormatter::new();
       b.iter(|| {
           formatter.format_error(&enhanced)
       });
   }
   ```
4. Fix clippy warnings:
   ```bash
   cargo clippy -p llmspell-bridge -- -D warnings
   # Focus on error handling modules
   cargo clippy -p llmspell-bridge --all-features -- \
       -W clippy::unwrap_used \
       -W clippy::expect_used \
       -W clippy::panic
   ```
5. Format and validate:
   ```bash
   cargo fmt -p llmspell-bridge -- --check
   cargo fmt -p llmspell-bridge
   ```
6. Documentation validation:
   ```bash
   cargo doc -p llmspell-bridge --no-deps --all-features
   # Check for broken links and missing docs
   cargo doc -p llmspell-bridge --all-features -- -D missing-docs
   ```

**Definition of Done:**
- [ ] All error enhancement tests pass
- [ ] Zero clippy warnings in error modules
- [ ] Code formatted consistently
- [ ] Performance overhead <10ms verified
- [ ] Error pattern coverage >80%
- [ ] Documentation complete for error APIs

---

## Phase 9.3: Interactive Debugging (Days 7-9)

### Task 9.3.1: Breakpoint System Implementation
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: Debug Team Lead

**Description**: Implement breakpoint system with file persistence and source comment support.

**Acceptance Criteria:**
- [ ] Breakpoints set via `.break file:line` command
- [ ] Breakpoints embedded as `--@breakpoint` comments
- [ ] Breakpoint conditions evaluated
- [ ] Breakpoints persist in `.llmspell-debug` file
- [ ] Breakpoint management commands work

**Implementation Steps:**
1. Create breakpoint manager:
   ```rust
   pub struct BreakpointManager {
       breakpoints: HashMap<PathBuf, HashSet<Breakpoint>>,
       debug_file: PathBuf,
   }
   
   pub struct Breakpoint {
       pub line: usize,
       pub condition: Option<String>,
       pub hit_count: usize,
       pub ignore_count: usize,
       pub enabled: bool,
       pub source: BreakpointSource,
   }
   
   pub enum BreakpointSource {
       Command,      // Set via REPL command
       SourceComment, // --@breakpoint in source
       DebugFile,    // From .llmspell-debug
   }
   ```
2. Parse source comments:
   ```rust
   impl BreakpointManager {
       pub fn scan_source_file(&mut self, path: &Path) -> Result<()> {
           let source = std::fs::read_to_string(path)?;
           
           for (line_no, line) in source.lines().enumerate() {
               // Check for --@breakpoint or --@bp
               if let Some(bp_comment) = Self::parse_breakpoint_comment(line) {
                   self.add_breakpoint(Breakpoint {
                       line: line_no + 1,
                       condition: bp_comment.condition,
                       source: BreakpointSource::SourceComment,
                       ..Default::default()
                   });
               }
           }
           
           Ok(())
       }
       
       fn parse_breakpoint_comment(line: &str) -> Option<BreakpointComment> {
           lazy_static! {
               static ref BP_REGEX: Regex = Regex::new(
                   r"--@(?:breakpoint|bp)(?:\s+if\s+(.+))?"
               ).unwrap();
           }
           
           BP_REGEX.captures(line).map(|caps| {
               BreakpointComment {
                   condition: caps.get(1).map(|m| m.as_str().to_string()),
               }
           })
       }
   }
   ```
3. Persist to `.llmspell-debug`:
   ```rust
   impl BreakpointManager {
       pub fn save_to_file(&self) -> Result<()> {
           let debug_data = DebugFileData {
               version: 1,
               breakpoints: self.serialize_breakpoints(),
               watches: vec![], // For future
           };
           
           let json = serde_json::to_string_pretty(&debug_data)?;
           std::fs::write(&self.debug_file, json)?;
           Ok(())
       }
       
       pub fn load_from_file(&mut self) -> Result<()> {
           if !self.debug_file.exists() {
               return Ok(());
           }
           
           let json = std::fs::read_to_string(&self.debug_file)?;
           let data: DebugFileData = serde_json::from_str(&json)?;
           
           self.deserialize_breakpoints(data.breakpoints);
           Ok(())
       }
   }
   ```
4. Install Lua hook for breakpoints:
   ```rust
   impl BreakpointManager {
       pub fn install_hook(&self, lua: &Lua) -> Result<()> {
           let manager = Arc::new(Mutex::new(self.clone()));
           
           lua.set_hook(
               HookTriggers {
                   every_line: true,
                   ..Default::default()
               },
               move |lua, debug| {
                   let info = debug.get_info();
                   let manager = manager.lock().unwrap();
                   
                   if let Some(bp) = manager.check_breakpoint(&info) {
                       if bp.should_break(lua) {
                           // Enter debug mode
                           return Err(DebugBreak::Breakpoint(bp));
                       }
                   }
                   
                   Ok(())
               }
           )?;
           
           Ok(())
       }
   }
   ```
5. Breakpoint commands:
   ```rust
   impl ReplCommands {
       pub fn handle_break_command(&mut self, args: &str) -> Result<()> {
           // Parse: .break file:line [if condition]
           let parts: Vec<&str> = args.splitn(2, " if ").collect();
           let location = parse_location(parts[0])?;
           let condition = parts.get(1).map(|s| s.to_string());
           
           self.breakpoint_manager.add_breakpoint(Breakpoint {
               file: location.file,
               line: location.line,
               condition,
               source: BreakpointSource::Command,
               ..Default::default()
           });
           
           println!("Breakpoint set at {}:{}", location.file, location.line);
           self.breakpoint_manager.save_to_file()?;
           Ok(())
       }
   }
   ```

**Definition of Done:**
- [ ] Breakpoints can be set via commands
- [ ] Source comment breakpoints detected
- [ ] Conditions evaluated correctly
- [ ] Persistence works across sessions
- [ ] Hook triggers at breakpoints

### Task 9.3.2: Step Debugging Implementation
**Priority**: CRITICAL  
**Estimated Time**: 5 hours  
**Assignee**: Debug Team

**Description**: Implement step, next, continue debugging commands.

**Acceptance Criteria:**
- [ ] `.step` steps into function calls
- [ ] `.next` steps over function calls
- [ ] `.continue` resumes execution
- [ ] `.up`/`.down` navigate call stack
- [ ] Current location displayed clearly

**Implementation Steps:**
1. Create debugger state machine:
   ```rust
   pub struct Debugger {
       state: DebugState,
       call_stack: Vec<CallFrame>,
       current_frame: usize,
       step_mode: StepMode,
   }
   
   pub enum DebugState {
       Running,
       Paused(PauseReason),
       Stepping,
   }
   
   pub enum StepMode {
       StepInto,    // Stop at next line
       StepOver,    // Stop at next line in same frame
       StepOut,     // Stop when returning from current frame
       Continue,    // Run until breakpoint
   }
   
   pub struct CallFrame {
       pub function_name: Option<String>,
       pub source_file: Option<PathBuf>,
       pub line: usize,
       pub locals: HashMap<String, Value>,
       pub depth: usize,
   }
   ```
2. Implement step commands:
   ```rust
   impl Debugger {
       pub fn step_into(&mut self) {
           self.step_mode = StepMode::StepInto;
           self.state = DebugState::Stepping;
       }
       
       pub fn step_over(&mut self) {
           self.step_mode = StepMode::StepOver;
           self.target_depth = self.current_frame().depth;
           self.state = DebugState::Stepping;
       }
       
       pub fn step_out(&mut self) {
           self.step_mode = StepMode::StepOut;
           self.target_depth = self.current_frame().depth.saturating_sub(1);
           self.state = DebugState::Stepping;
       }
       
       pub fn continue_execution(&mut self) {
           self.step_mode = StepMode::Continue;
           self.state = DebugState::Running;
       }
   }
   ```
3. Update Lua hook for stepping:
   ```rust
   impl Debugger {
       pub fn should_pause(&self, info: &DebugInfo) -> bool {
           match self.step_mode {
               StepMode::StepInto => true,
               StepMode::StepOver => info.depth <= self.target_depth,
               StepMode::StepOut => info.depth < self.target_depth,
               StepMode::Continue => false,
           }
       }
       
       pub fn update_hook(&self, lua: &Lua) -> Result<()> {
           let debugger = Arc::new(Mutex::new(self.clone()));
           
           lua.set_hook(
               HookTriggers {
                   every_line: true,
                   on_calls: true,
                   on_returns: true,
               },
               move |lua, debug| {
                   let mut debugger = debugger.lock().unwrap();
                   let info = debug.get_info();
                   
                   // Update call stack
                   match debug.event() {
                       DebugEvent::Call => debugger.push_frame(&info),
                       DebugEvent::Return => debugger.pop_frame(),
                       DebugEvent::Line => {
                           if debugger.should_pause(&info) {
                               debugger.pause_at(&info, lua)?;
                           }
                       }
                       _ => {}
                   }
                   
                   Ok(())
               }
           )?;
           
           Ok(())
       }
   }
   ```
4. Stack navigation:
   ```rust
   impl Debugger {
       pub fn move_up(&mut self) -> Result<()> {
           if self.current_frame > 0 {
               self.current_frame -= 1;
               self.display_frame();
           } else {
               println!("Already at top of stack");
           }
           Ok(())
       }
       
       pub fn move_down(&mut self) -> Result<()> {
           if self.current_frame < self.call_stack.len() - 1 {
               self.current_frame += 1;
               self.display_frame();
           } else {
               println!("Already at bottom of stack");
           }
           Ok(())
       }
       
       pub fn display_frame(&self) {
           let frame = &self.call_stack[self.current_frame];
           println!("#{} {} at {}:{}",
               self.current_frame,
               frame.function_name.as_deref().unwrap_or("<main>"),
               frame.source_file.as_ref().map(|p| p.display()).unwrap_or("?"),
               frame.line
           );
       }
   }
   ```
5. Test stepping through complex code

**Definition of Done:**
- [ ] Step commands work correctly
- [ ] Call stack tracked accurately
- [ ] Frame navigation functional
- [ ] Current location displayed
- [ ] Stepping performance acceptable

### Task 9.3.3: Variable Inspection with Lazy Expansion
**Priority**: HIGH  
**Estimated Time**: 5 hours  
**Assignee**: Debug Team

**Description**: Implement interactive variable inspection with lazy expansion for complex objects.

**Acceptance Criteria:**
- [ ] `.locals` shows all local variables
- [ ] `.print var` inspects specific variable
- [ ] Complex objects expand interactively
- [ ] Cycles detected and handled
- [ ] `.watch expr` adds watch expressions

**Implementation Steps:**
1. Create variable inspector:
   ```rust
   pub struct VariableInspector {
       max_depth: usize,
       max_items: usize,
       expansion_cache: HashMap<String, ExpandedValue>,
   }
   
   pub struct ExpandedValue {
       pub name: String,
       pub value: Value,
       pub type_name: String,
       pub expandable: bool,
       pub expanded: bool,
       pub children: Option<Vec<ExpandedValue>>,
   }
   ```
2. Implement inspection logic:
   ```rust
   impl VariableInspector {
       pub fn inspect_value(&mut self, name: &str, value: &Value) -> ExpandedValue {
           ExpandedValue {
               name: name.to_string(),
               value: value.clone(),
               type_name: Self::get_type_name(value),
               expandable: Self::is_expandable(value),
               expanded: false,
               children: None,
           }
       }
       
       pub fn expand_value(&mut self, value: &mut ExpandedValue) -> Result<()> {
           if !value.expandable || value.expanded {
               return Ok(());
           }
           
           value.children = Some(match &value.value {
               Value::Table(table) => {
                   let mut children = Vec::new();
                   let mut count = 0;
                   
                   for (key, val) in table.iter() {
                       if count >= self.max_items {
                           children.push(ExpandedValue::truncation_marker(
                               table.len() - count
                           ));
                           break;
                       }
                       
                       let key_str = Self::format_key(&key);
                       children.push(self.inspect_value(&key_str, &val));
                       count += 1;
                   }
                   
                   children
               }
               Value::UserData(ud) => {
                   // Inspect userdata fields
                   Self::inspect_userdata(ud)?
               }
               _ => vec![],
           });
           
           value.expanded = true;
           Ok(())
       }
   }
   ```
3. Interactive display:
   ```rust
   impl VariableInspector {
       pub fn interactive_inspect(&mut self, name: &str, value: &Value) -> Result<()> {
           let mut root = self.inspect_value(name, value);
           let mut stack = vec![&mut root];
           
           loop {
               self.display_tree(&root, 0);
               
               let action = inquire::Select::new(
                   "Inspect:",
                   vec!["Expand", "Collapse", "Copy Path", "Done"],
               ).prompt()?;
               
               match action {
                   "Expand" => {
                       let path = self.select_expandable_path(&root)?;
                       if let Some(node) = self.find_node_mut(&mut root, &path) {
                           self.expand_value(node)?;
                       }
                   }
                   "Collapse" => {
                       let path = self.select_expanded_path(&root)?;
                       if let Some(node) = self.find_node_mut(&mut root, &path) {
                           node.expanded = false;
                       }
                   }
                   "Copy Path" => {
                       let path = self.select_any_path(&root)?;
                       println!("Path copied: {}", path);
                   }
                   "Done" => break,
               }
           }
           
           Ok(())
       }
       
       fn display_tree(&self, node: &ExpandedValue, depth: usize) {
           let indent = "  ".repeat(depth);
           let marker = if node.expandable {
               if node.expanded { "▼" } else { "▶" }
           } else {
               "·"
           };
           
           println!("{}{} {} = {} ({})",
               indent,
               marker,
               node.name,
               self.format_value_preview(&node.value),
               node.type_name
           );
           
           if let Some(children) = &node.children {
               for child in children {
                   self.display_tree(child, depth + 1);
               }
           }
       }
   }
   ```
4. Watch expressions:
   ```rust
   pub struct WatchManager {
       watches: Vec<WatchExpression>,
   }
   
   pub struct WatchExpression {
       pub id: usize,
       pub expression: String,
       pub last_value: Option<Value>,
       pub changed: bool,
   }
   
   impl WatchManager {
       pub fn add_watch(&mut self, expr: &str) -> usize {
           let id = self.watches.len();
           self.watches.push(WatchExpression {
               id,
               expression: expr.to_string(),
               last_value: None,
               changed: false,
           });
           id
       }
       
       pub fn evaluate_watches(&mut self, lua: &Lua) -> Result<()> {
           for watch in &mut self.watches {
               match lua.load(&watch.expression).eval::<Value>() {
                   Ok(value) => {
                       watch.changed = watch.last_value.as_ref()
                           .map_or(true, |old| !values_equal(old, &value));
                       watch.last_value = Some(value);
                   }
                   Err(e) => {
                       println!("Watch '{}' error: {}", watch.expression, e);
                   }
               }
           }
           Ok(())
       }
       
       pub fn display_watches(&self) {
           for watch in &self.watches {
               let marker = if watch.changed { "*" } else { " " };
               println!("{} [{}] {} = {}",
                   marker,
                   watch.id,
                   watch.expression,
                   watch.last_value.as_ref()
                       .map(|v| format_value(v))
                       .unwrap_or_else(|| "<not evaluated>".to_string())
               );
           }
       }
   }
   ```
5. Test with complex data structures

**Definition of Done:**
- [ ] Local variables displayed correctly
- [ ] Variable inspection works
- [ ] Lazy expansion functional
- [ ] Cycle detection prevents infinite loops
- [ ] Watch expressions evaluated

### Task 9.3.4: Section 9.3 Testing and Code Quality
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: QA Team

**Description**: Comprehensive testing and code quality validation for Interactive Debugging.

**Acceptance Criteria:**
- [ ] Breakpoint system fully tested
- [ ] Step debugging tests cover all scenarios
- [ ] Variable inspection tests validate correctness
- [ ] Zero clippy warnings in debug code
- [ ] Code formatted and documented
- [ ] Debug performance <1ms overhead

**Implementation Steps:**
1. Write comprehensive debug tests:
   ```rust
   #[cfg(test)]
   mod debug_tests {
       use super::*;
       
       #[test]
       fn test_breakpoint_parsing() {
           let mgr = BreakpointManager::new();
           
           // Test comment parsing
           let comment = "--@breakpoint if x > 10";
           let bp = mgr.parse_breakpoint_comment(comment);
           assert!(bp.is_some());
           assert_eq!(bp.unwrap().condition, Some("x > 10".to_string()));
       }
       
       #[test]
       fn test_breakpoint_persistence() {
           let mut mgr = BreakpointManager::new();
           mgr.add_breakpoint(Breakpoint {
               line: 42,
               file: "test.lua".into(),
               ..Default::default()
           });
           
           mgr.save_to_file().unwrap();
           
           let mut loaded = BreakpointManager::new();
           loaded.load_from_file().unwrap();
           assert!(loaded.has_breakpoint("test.lua", 42));
       }
       
       #[test]
       fn test_step_debugging_state_machine() {
           let mut debugger = Debugger::new();
           
           debugger.step_into();
           assert_eq!(debugger.step_mode, StepMode::StepInto);
           
           debugger.step_over();
           assert_eq!(debugger.step_mode, StepMode::StepOver);
           
           debugger.continue_execution();
           assert_eq!(debugger.state, DebugState::Running);
       }
       
       #[test]
       fn test_variable_inspection_cycles() {
           let mut inspector = VariableInspector::new();
           
           // Create circular reference
           let table = lua.create_table().unwrap();
           table.set("self", table.clone()).unwrap();
           
           let expanded = inspector.inspect_value("circular", &Value::Table(table));
           assert!(expanded.expandable);
           
           // Should handle cycles without stack overflow
           inspector.expand_value(&mut expanded).unwrap();
       }
       
       #[test]
       fn test_watch_expression_evaluation() {
           let mut watch_mgr = WatchManager::new();
           
           let id = watch_mgr.add_watch("x + y");
           lua.globals().set("x", 10).unwrap();
           lua.globals().set("y", 20).unwrap();
           
           watch_mgr.evaluate_watches(&lua).unwrap();
           
           let watch = &watch_mgr.watches[id];
           assert_eq!(watch.last_value, Some(Value::Integer(30)));
       }
   }
   ```
2. Integration tests for debugging scenarios:
   ```rust
   #[tokio::test]
   async fn test_debug_session_complete() {
       let mut repl = create_test_repl_with_debugger().await;
       
       // Set breakpoint
       repl.process_input(".break test.lua:10").await.unwrap();
       
       // Run script that hits breakpoint
       repl.process_input("dofile('test.lua')").await.unwrap();
       
       // Verify paused at breakpoint
       assert_eq!(repl.debugger.state, DebugState::Paused);
       
       // Step and inspect
       repl.process_input(".locals").await.unwrap();
       repl.process_input(".step").await.unwrap();
       repl.process_input(".continue").await.unwrap();
   }
   
   #[tokio::test]
   async fn test_conditional_breakpoints() {
       let mut debugger = create_test_debugger();
       
       debugger.add_breakpoint(Breakpoint {
           line: 10,
           condition: Some("counter > 5".to_string()),
           ..Default::default()
       });
       
       // Should not break when condition false
       lua.globals().set("counter", 3).unwrap();
       assert!(!debugger.should_break_at(10));
       
       // Should break when condition true
       lua.globals().set("counter", 7).unwrap();
       assert!(debugger.should_break_at(10));
   }
   ```
3. Performance benchmarks:
   ```rust
   #[bench]
   fn bench_breakpoint_checking(b: &mut Bencher) {
       let mgr = create_manager_with_many_breakpoints();
       b.iter(|| {
           mgr.check_breakpoint(&debug_info)
       });
   }
   
   #[bench]
   fn bench_variable_inspection(b: &mut Bencher) {
       let complex_table = create_deeply_nested_table();
       let mut inspector = VariableInspector::new();
       b.iter(|| {
           inspector.inspect_value("complex", &complex_table)
       });
   }
   ```
4. Clippy strict mode:
   ```bash
   # Debug-specific clippy checks
   cargo clippy -p llmspell-bridge --all-features -- \
       -D warnings \
       -W clippy::cognitive_complexity \
       -W clippy::missing_const_for_fn \
       -W clippy::missing_errors_doc
   ```
5. Format validation:
   ```bash
   cargo fmt -p llmspell-bridge --all -- --check
   cargo fmt -p llmspell-cli --all -- --check
   ```
6. Debug-specific documentation:
   ```rust
   /// Comprehensive documentation for debug features
   /// Including examples and common patterns
   ```

**Definition of Done:**
- [ ] All debug tests pass with 100% coverage
- [ ] Integration tests cover all debug scenarios
- [ ] Zero clippy warnings in debug modules
- [ ] Performance overhead <1ms verified
- [ ] Code formatted consistently
- [ ] Debug API fully documented

---

## Phase 9.4: Development Experience (Days 10-11)

### Task 9.4.1: Hot Reload Implementation
**Priority**: HIGH  
**Estimated Time**: 5 hours  
**Assignee**: Dev Experience Team

**Description**: Implement both automatic and manual hot reload capabilities.

**Acceptance Criteria:**
- [ ] File watcher detects changes automatically
- [ ] `.reload` command triggers manual reload
- [ ] State preserved across reloads
- [ ] Validation before reload
- [ ] Error recovery without losing session

**Implementation Steps:**
1. Create hot reload manager:
   ```rust
   use notify::{Watcher, RecursiveMode, Event};
   
   pub struct HotReloadManager {
       watcher: Option<RecommendedWatcher>,
       watched_files: HashSet<PathBuf>,
       runtime: Arc<ScriptRuntime>,
       state_snapshot: Option<StateSnapshot>,
       auto_reload: bool,
   }
   ```
2. Implement file watching:
   ```rust
   impl HotReloadManager {
       pub fn start_watching(&mut self, path: &Path) -> Result<()> {
           let (tx, rx) = channel();
           let mut watcher = notify::recommended_watcher(tx)?;
           
           watcher.watch(path, RecursiveMode::NonRecursive)?;
           self.watched_files.insert(path.to_path_buf());
           
           // Spawn handler thread
           let manager = Arc::new(Mutex::new(self.clone()));
           thread::spawn(move || {
               for event in rx {
                   if let Ok(Event::Modify(_)) = event {
                       let manager = manager.lock().unwrap();
                       if manager.auto_reload {
                           manager.trigger_reload();
                       } else {
                           println!("File changed. Use .reload to reload.");
                       }
                   }
               }
           });
           
           self.watcher = Some(watcher);
           Ok(())
       }
   }
   ```
3. State preservation:
   ```rust
   impl HotReloadManager {
       pub async fn reload_with_state(&mut self, path: &Path) -> Result<()> {
           // Save current state
           let snapshot = self.capture_state().await?;
           
           // Read and validate new script
           let script = fs::read_to_string(path).await?;
           if let Err(e) = self.validate_script(&script).await {
               println!("Validation failed: {}. Reload cancelled.", e);
               return Ok(());
           }
           
           // Clear old definitions but keep state
           self.runtime.clear_definitions().await?;
           
           // Execute new script
           match self.runtime.execute_script(&script).await {
               Ok(_) => {
                   // Restore state
                   self.restore_state(snapshot).await?;
                   println!("✓ Script reloaded successfully");
               }
               Err(e) => {
                   // Restore previous version
                   self.restore_from_backup().await?;
                   println!("✗ Reload failed: {}. Previous version restored.", e);
               }
           }
           
           Ok(())
       }
       
       async fn capture_state(&self) -> Result<StateSnapshot> {
           Ok(StateSnapshot {
               variables: self.runtime.get_globals().await?,
               breakpoints: self.breakpoint_manager.export(),
               watches: self.watch_manager.export(),
               history: self.history_manager.recent(10),
           })
       }
       
       async fn restore_state(&mut self, snapshot: StateSnapshot) -> Result<()> {
           // Restore globals
           for (name, value) in snapshot.variables {
               self.runtime.set_global(&name, value).await?;
           }
           
           // Restore debug state
           self.breakpoint_manager.import(snapshot.breakpoints);
           self.watch_manager.import(snapshot.watches);
           
           Ok(())
       }
   }
   ```
4. Manual reload command:
   ```rust
   impl ReplCommands {
       pub async fn handle_reload(&mut self, args: Option<&str>) -> Result<()> {
           let path = if let Some(p) = args {
               PathBuf::from(p)
           } else if let Some(p) = &self.current_script {
               p.clone()
           } else {
               return Err(anyhow!("No script to reload. Specify a path."));
           };
           
           self.hot_reload_manager.reload_with_state(&path).await?;
           Ok(())
       }
   }
   ```
5. Test with various reload scenarios

**Definition of Done:**
- [ ] File watching works
- [ ] Manual reload command functional
- [ ] State preserved correctly
- [ ] Validation prevents bad reloads
- [ ] Error recovery works

### Task 9.4.2: Script Validation System
**Priority**: MEDIUM  
**Estimated Time**: 4 hours  
**Assignee**: Dev Experience Team

**Description**: Implement comprehensive script validation before execution.

**Acceptance Criteria:**
- [ ] Syntax errors caught before execution
- [ ] Undefined global warnings
- [ ] Type inference warnings
- [ ] Common mistake detection
- [ ] `.validate` command works

**Implementation Steps:**
1. Create validator:
   ```rust
   pub struct ScriptValidator {
       lua: Lua,
       known_globals: HashSet<String>,
       type_hints: HashMap<String, TypeHint>,
   }
   
   pub struct ValidationReport {
       pub errors: Vec<ValidationError>,
       pub warnings: Vec<ValidationWarning>,
       pub info: Vec<ValidationInfo>,
   }
   ```
2. Syntax validation:
   ```rust
   impl ScriptValidator {
       pub fn validate_syntax(&self, script: &str) -> Result<(), ValidationError> {
           // Try to compile without executing
           match self.lua.load(script).into_function() {
               Ok(_) => Ok(()),
               Err(e) => Err(ValidationError::Syntax {
                   message: e.to_string(),
                   location: Self::extract_error_location(&e),
               }),
           }
       }
   }
   ```
3. Static analysis:
   ```rust
   impl ScriptValidator {
       pub fn analyze_script(&mut self, script: &str) -> ValidationReport {
           let mut report = ValidationReport::default();
           
           // Parse to AST (using full-moon or similar)
           let ast = match parse_lua(script) {
               Ok(ast) => ast,
               Err(e) => {
                   report.errors.push(ValidationError::Parse(e));
                   return report;
               }
           };
           
           // Check undefined globals
           for global in find_global_accesses(&ast) {
               if !self.known_globals.contains(&global) {
                   report.warnings.push(ValidationWarning::UndefinedGlobal {
                       name: global.clone(),
                       suggestion: self.find_similar_global(&global),
                   });
               }
           }
           
           // Check common mistakes
           for issue in detect_common_issues(&ast) {
               report.warnings.push(issue);
           }
           
           // Basic type checking
           for type_issue in self.check_types(&ast) {
               report.warnings.push(type_issue);
           }
           
           report
       }
   }
   ```
4. Common issue detection:
   ```rust
   fn detect_common_issues(ast: &Ast) -> Vec<ValidationWarning> {
       let mut warnings = Vec::new();
       
       // Unused variables
       for var in find_unused_variables(ast) {
           warnings.push(ValidationWarning::UnusedVariable {
               name: var.name,
               location: var.location,
           });
       }
       
       // Shadowed variables
       for shadow in find_shadowed_variables(ast) {
           warnings.push(ValidationWarning::ShadowedVariable {
               name: shadow.name,
               outer_location: shadow.outer,
               inner_location: shadow.inner,
           });
       }
       
       // Unreachable code
       for unreachable in find_unreachable_code(ast) {
           warnings.push(ValidationWarning::UnreachableCode {
               location: unreachable,
           });
       }
       
       // Possible nil access
       for nil_access in detect_possible_nil_access(ast) {
           warnings.push(ValidationWarning::PossibleNilAccess {
               variable: nil_access.variable,
               location: nil_access.location,
           });
       }
       
       warnings
   }
   ```
5. Validation command:
   ```rust
   impl ReplCommands {
       pub fn handle_validate(&mut self, path: &str) -> Result<()> {
           let script = std::fs::read_to_string(path)?;
           let report = self.validator.analyze_script(&script);
           
           // Display report
           if !report.errors.is_empty() {
               println!("{}:", "Errors".red().bold());
               for error in &report.errors {
                   println!("  {}", error);
               }
           }
           
           if !report.warnings.is_empty() {
               println!("{}:", "Warnings".yellow().bold());
               for warning in &report.warnings {
                   println!("  {}", warning);
               }
           }
           
           if report.errors.is_empty() && report.warnings.is_empty() {
               println!("{}", "✓ No issues found".green());
           }
           
           Ok(())
       }
   }
   ```

**Definition of Done:**
- [ ] Syntax validation works
- [ ] Static analysis functional
- [ ] Common issues detected
- [ ] Validation report clear
- [ ] Performance acceptable

### Task 9.4.3: Full Performance Profiling
**Priority**: HIGH  
**Estimated Time**: 6 hours  
**Assignee**: Dev Experience Team

**Description**: Implement comprehensive profiling with timing, memory, and flamegraph generation.

**Acceptance Criteria:**
- [ ] `.profile start/stop` commands work
- [ ] Execution timing tracked
- [ ] Memory allocation tracked
- [ ] Flamegraphs generated
- [ ] External tool integration (perf, valgrind)

**Implementation Steps:**
1. Create profiler system:
   ```rust
   pub struct Profiler {
       timing: TimingProfiler,
       memory: MemoryProfiler,
       flamegraph: FlamegraphBuilder,
       external: ExternalProfiler,
   }
   
   pub struct ProfilingSession {
       id: Uuid,
       start_time: Instant,
       samples: Vec<ProfileSample>,
       memory_snapshots: Vec<MemorySnapshot>,
   }
   ```
2. Timing profiler:
   ```rust
   impl TimingProfiler {
       pub fn start_profiling(&mut self) -> ProfileHandle {
           let handle = ProfileHandle::new();
           
           // Install Lua hook for sampling
           self.install_sampling_hook();
           
           handle
       }
       
       fn install_sampling_hook(&self) {
           let profiler = Arc::new(Mutex::new(self.clone()));
           
           self.lua.set_hook(
               HookTriggers {
                   every_nth_instruction: Some(1000),
               },
               move |lua, debug| {
                   let mut profiler = profiler.lock().unwrap();
                   let info = debug.get_info();
                   
                   profiler.record_sample(ProfileSample {
                       timestamp: Instant::now(),
                       function: info.name.clone(),
                       source: info.source.clone(),
                       line: info.current_line,
                   });
                   
                   Ok(())
               }
           );
       }
       
       pub fn generate_report(&self) -> TimingReport {
           let mut function_times = HashMap::new();
           
           for window in self.samples.windows(2) {
               let duration = window[1].timestamp - window[0].timestamp;
               let entry = function_times
                   .entry(window[0].function.clone())
                   .or_insert(FunctionTiming::default());
               
               entry.total_time += duration;
               entry.call_count += 1;
           }
           
           TimingReport {
               total_time: self.end_time - self.start_time,
               function_times,
               hotspots: self.find_hotspots(&function_times),
           }
       }
   }
   ```
3. Memory profiler:
   ```rust
   impl MemoryProfiler {
       pub fn track_allocations(&mut self) -> Result<()> {
           // Use jemalloc or mimalloc with profiling
           #[cfg(feature = "jemalloc")]
           {
               jemalloc_ctl::prof::active::write(true)?;
           }
           
           // Periodic snapshots
           self.snapshot_timer = Some(tokio::spawn(async move {
               loop {
                   tokio::time::sleep(Duration::from_millis(100)).await;
                   self.capture_snapshot();
               }
           }));
           
           Ok(())
       }
       
       pub fn capture_snapshot(&mut self) {
           let snapshot = MemorySnapshot {
               timestamp: Instant::now(),
               heap_size: self.get_heap_size(),
               lua_memory: self.get_lua_memory(),
               allocations: self.get_allocation_profile(),
           };
           
           self.snapshots.push(snapshot);
       }
       
       pub fn generate_report(&self) -> MemoryReport {
           MemoryReport {
               peak_memory: self.snapshots.iter()
                   .map(|s| s.heap_size)
                   .max()
                   .unwrap_or(0),
               allocation_sites: self.analyze_allocation_sites(),
               leak_candidates: self.detect_potential_leaks(),
           }
       }
   }
   ```
4. Flamegraph generation:
   ```rust
   impl FlamegraphBuilder {
       pub fn build_flamegraph(&self, samples: &[ProfileSample]) -> Result<String> {
           // Build stack traces
           let mut stacks = Vec::new();
           
           for sample in samples {
               let stack = self.build_stack_trace(sample);
               stacks.push(stack);
           }
           
           // Generate flamegraph SVG
           let mut options = flamegraph::Options::default();
           options.title = "LLMSpell Script Profile".to_string();
           
           let mut buffer = Vec::new();
           flamegraph::from_lines(
               &mut options,
               stacks.iter().map(|s| s.as_str()),
               &mut buffer,
           )?;
           
           Ok(String::from_utf8(buffer)?)
       }
       
       fn build_stack_trace(&self, sample: &ProfileSample) -> String {
           // Format: function1;function2;function3 count
           let stack = self.get_call_stack_at(sample.timestamp);
           let formatted = stack.join(";");
           format!("{} 1", formatted)
       }
   }
   ```
5. External tool integration:
   ```rust
   impl ExternalProfiler {
       pub fn start_perf_record(&self, pid: u32) -> Result<Child> {
           Command::new("perf")
               .args(&["record", "-F", "99", "-p", &pid.to_string()])
               .spawn()
               .map_err(Into::into)
       }
       
       pub fn generate_perf_report(&self) -> Result<String> {
           let output = Command::new("perf")
               .args(&["report", "--stdio"])
               .output()?;
           
           Ok(String::from_utf8_lossy(&output.stdout).to_string())
       }
       
       pub fn run_with_valgrind(&self, script: &str) -> Result<String> {
           let output = Command::new("valgrind")
               .args(&[
                   "--tool=massif",
                   "--massif-out-file=massif.out",
                   "llmspell",
                   "run",
                   script,
               ])
               .output()?;
           
           // Parse massif output
           self.parse_massif_output("massif.out")
       }
   }
   ```

**Definition of Done:**
- [ ] Profiling sessions work
- [ ] Timing profiler accurate
- [ ] Memory tracking functional
- [ ] Flamegraphs generated
- [ ] External tools integrated

### Task 9.4.4: Section 9.4 Testing and Code Quality
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: QA Team

**Description**: Comprehensive testing and code quality validation for Development Experience features.

**Acceptance Criteria:**
- [ ] All hot reload functionality tested
- [ ] Script validation tests comprehensive
- [ ] Performance profiling tested end-to-end
- [ ] Integration tests for file watching
- [ ] Zero clippy warnings across all components
- [ ] Code formatted consistently
- [ ] Memory leak tests for profiling components

**Implementation Steps:**
1. Unit tests for hot reload:
   ```rust
   #[cfg(test)]
   mod hot_reload_tests {
       use super::*;
       use std::fs;
       use tempfile::TempDir;
       
       #[tokio::test]
       async fn test_hot_reload_file_change() {
           let temp_dir = TempDir::new().unwrap();
           let script_path = temp_dir.path().join("test.lua");
           
           fs::write(&script_path, "print('version 1')").unwrap();
           
           let mut manager = HotReloadManager::new();
           manager.watch_file(&script_path).await.unwrap();
           
           // Simulate file change
           fs::write(&script_path, "print('version 2')").unwrap();
           
           // Should trigger reload
           let reloaded = manager.wait_for_reload(Duration::from_secs(1)).await;
           assert!(reloaded);
       }
       
       #[tokio::test]
       async fn test_state_preservation_during_reload() {
           let runtime = create_test_runtime().await;
           let mut manager = HotReloadManager::with_runtime(runtime);
           
           // Set some state
           manager.runtime.set_global("test_var", "preserved_value").unwrap();
           
           // Reload
           manager.reload_script("print('reloaded')").await.unwrap();
           
           // Check state preserved
           let value: String = manager.runtime.get_global("test_var").unwrap();
           assert_eq!(value, "preserved_value");
       }
   }
   ```

2. Script validation tests:
   ```rust
   #[cfg(test)]
   mod validation_tests {
       #[test]
       fn test_syntax_validation() {
           let validator = ScriptValidator::new();
           
           let valid_script = "print('hello')";
           assert!(validator.validate_syntax(valid_script).is_ok());
           
           let invalid_script = "print('unclosed";
           assert!(validator.validate_syntax(invalid_script).is_err());
       }
       
       #[test]
       fn test_api_validation() {
           let validator = ScriptValidator::with_api_check();
           
           let valid_api = "local result = Tool.invoke('calculator', {operation = 'add'})";
           assert!(validator.validate_api_usage(valid_api).is_ok());
           
           let invalid_api = "local result = NonexistentAPI.call()";
           assert!(validator.validate_api_usage(invalid_api).is_err());
       }
   }
   ```

3. Profiling tests:
   ```rust
   #[cfg(test)]
   mod profiling_tests {
       #[tokio::test]
       async fn test_timing_profiler_accuracy() {
           let mut profiler = TimingProfiler::new();
           profiler.start_profiling().await;
           
           // Run some measured work
           tokio::time::sleep(Duration::from_millis(100)).await;
           
           let report = profiler.stop_and_report().await;
           
           // Should be approximately 100ms (with some tolerance)
           assert!(report.total_time >= Duration::from_millis(95));
           assert!(report.total_time <= Duration::from_millis(105));
       }
       
       #[tokio::test]
       async fn test_memory_profiler_tracking() {
           let mut profiler = MemoryProfiler::new();
           profiler.start_tracking().await;
           
           // Allocate some memory
           let _data = vec![0u8; 1024 * 1024]; // 1MB
           
           let snapshot = profiler.take_snapshot().await;
           assert!(snapshot.heap_size > 1024 * 1024);
           
           profiler.stop_tracking().await;
       }
   }
   ```

4. Integration tests:
   ```rust
   #[cfg(test)]
   mod integration_tests {
       #[tokio::test]
       async fn test_full_development_workflow() {
           let temp_dir = create_temp_project().await;
           let session = ReplSession::new_in_dir(&temp_dir).await.unwrap();
           
           // Start with hot reload enabled
           session.execute_command(".reload auto").await.unwrap();
           
           // Create and run initial script
           let script_path = temp_dir.join("main.lua");
           fs::write(&script_path, "print('initial version')").unwrap();
           
           session.execute_command("dofile('main.lua')").await.unwrap();
           
           // Modify script and verify reload
           fs::write(&script_path, "print('modified version')").unwrap();
           
           // Wait for auto-reload
           tokio::time::sleep(Duration::from_millis(200)).await;
           
           // Verify new version loaded
           let output = session.execute_command("dofile('main.lua')").await.unwrap();
           assert!(output.contains("modified version"));
       }
   }
   ```

5. Performance benchmarks:
   ```rust
   #[cfg(test)]
   mod performance_tests {
       #[bench]
       fn bench_hot_reload_time(b: &mut Bencher) {
           let rt = tokio::runtime::Runtime::new().unwrap();
           
           b.iter(|| {
               rt.block_on(async {
                   let mut manager = HotReloadManager::new();
                   manager.reload_script("print('benchmark')").await.unwrap();
               })
           });
       }
       
       #[bench] 
       fn bench_profiler_overhead(b: &mut Bencher) {
           let rt = tokio::runtime::Runtime::new().unwrap();
           
           b.iter(|| {
               rt.block_on(async {
                   let mut profiler = TimingProfiler::new();
                   profiler.start_profiling().await;
                   // Some minimal work
                   tokio::time::sleep(Duration::from_nanos(1)).await;
                   profiler.stop_and_report().await;
               })
           });
       }
   }
   ```

6. Code quality validation:
   ```bash
   # Zero clippy warnings
   cargo clippy --all-features --all-targets -- -D warnings
   
   # Consistent formatting
   cargo fmt --all -- --check
   
   # Documentation coverage
   RUSTDOCFLAGS="-D missing_docs" cargo doc --all-features --no-deps
   
   # Memory leak detection
   valgrind --tool=memcheck --leak-check=full \
       ./target/debug/llmspell run test_profiling_script.lua
   ```

**Definition of Done:**
- [ ] Hot reload tests pass with state preservation
- [ ] Script validation catches syntax/API errors
- [ ] Profiling accuracy within 5% tolerance
- [ ] Integration tests cover full dev workflow
- [ ] Zero clippy warnings in all development experience code
- [ ] All code formatted consistently
- [ ] Performance benchmarks establish baseline
- [ ] Memory leak tests pass

---

## Phase 9.5: Advanced Features (Days 12-14)

### Task 9.5.1: Hook Introspection Integration
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: Integration Team

**Description**: Integrate with Phase 4 hook system for debugging and monitoring.

**Acceptance Criteria:**
- [ ] `.hooks list` shows all active hooks
- [ ] `.hooks trace` traces hook execution
- [ ] Event stream visualization works
- [ ] Performance metrics displayed
- [ ] Circuit breaker status shown

**Implementation Steps:**
1. Create hook inspector:
   ```rust
   pub struct HookInspector {
       hook_manager: Arc<HookManager>, // From Phase 4
       event_bus: Arc<EventBus>,       // From Phase 4
   }
   
   impl HookInspector {
       pub fn list_hooks(&self) -> Vec<HookInfo> {
           self.hook_manager.get_registered_hooks()
               .map(|hook| HookInfo {
                   name: hook.name(),
                   type_: hook.hook_type(),
                   priority: hook.priority(),
                   enabled: hook.is_enabled(),
                   execution_count: hook.stats().execution_count,
                   average_time: hook.stats().average_time,
               })
               .collect()
       }
       
       pub fn trace_hooks(&mut self, enabled: bool) {
           self.hook_manager.set_tracing(enabled);
           if enabled {
               println!("Hook tracing enabled. Hooks will be logged as they execute.");
           }
       }
   }
   ```
2. Event stream monitor:
   ```rust
   impl EventStreamMonitor {
       pub async fn start_monitoring(&mut self) -> Result<()> {
           let mut receiver = self.event_bus.subscribe();
           
           tokio::spawn(async move {
               while let Ok(event) = receiver.recv().await {
                   self.display_event(&event);
               }
           });
           
           Ok(())
       }
       
       fn display_event(&self, event: &Event) {
           let timestamp = event.timestamp.format("%H:%M:%S%.3f");
           let colored_type = match event.severity {
               Severity::Error => event.type_.red(),
               Severity::Warning => event.type_.yellow(),
               Severity::Info => event.type_.blue(),
               _ => event.type_.normal(),
           };
           
           println!("[{}] {} - {}", timestamp, colored_type, event.message);
           
           if self.verbose {
               for (key, value) in &event.metadata {
                   println!("  {} = {}", key.dim(), value);
               }
           }
       }
   }
   ```
3. Performance monitor integration:
   ```rust
   impl PerformanceMonitor {
       pub fn display_metrics(&self) {
           let metrics = self.hook_manager.get_performance_metrics();
           
           println!("Hook Performance Metrics:");
           println!("  Total executions: {}", metrics.total_executions);
           println!("  Average time: {:.2}ms", metrics.average_time_ms);
           println!("  P95 time: {:.2}ms", metrics.p95_time_ms);
           println!("  P99 time: {:.2}ms", metrics.p99_time_ms);
           
           if let Some(cb_status) = metrics.circuit_breaker_status {
               println!("\nCircuit Breaker Status:");
               println!("  State: {}", cb_status.state);
               println!("  Failures: {}/{}", 
                   cb_status.failure_count,
                   cb_status.failure_threshold
               );
               println!("  Last trip: {}", 
                   cb_status.last_trip_time
                       .map(|t| t.format("%H:%M:%S").to_string())
                       .unwrap_or_else(|| "Never".to_string())
               );
           }
       }
   }
   ```

**Definition of Done:**
- [ ] Hook listing works
- [ ] Hook tracing functional
- [ ] Event stream displayed
- [ ] Performance metrics shown
- [ ] Circuit breaker monitored

### Task 9.5.2: Debug Protocol Implementation
**Priority**: HIGH  
**Estimated Time**: 8 hours  
**Assignee**: Protocol Team

**Description**: Implement both DAP and LSP protocols for IDE integration.

**Acceptance Criteria:**
- [ ] DAP server implements core debug commands
- [ ] LSP server provides completion and diagnostics
- [ ] VS Code extension connects successfully
- [ ] Protocol compliance validated
- [ ] Multi-client support works

**Implementation Steps:**
1. DAP server implementation:
   ```rust
   use dap::{Server, Request, Response};
   
   pub struct DapServer {
       debugger: Arc<Mutex<Debugger>>,
       server: Server,
   }
   
   impl DapServer {
       pub async fn start(&mut self, port: u16) -> Result<()> {
           self.server.listen(port).await?;
           
           while let Some(request) = self.server.accept().await? {
               self.handle_request(request).await?;
           }
           
           Ok(())
       }
       
       async fn handle_request(&mut self, req: Request) -> Result<Response> {
           match req.command.as_str() {
               "initialize" => self.handle_initialize(req).await,
               "setBreakpoints" => self.handle_set_breakpoints(req).await,
               "continue" => self.handle_continue(req).await,
               "next" => self.handle_next(req).await,
               "stepIn" => self.handle_step_in(req).await,
               "stepOut" => self.handle_step_out(req).await,
               "evaluate" => self.handle_evaluate(req).await,
               "variables" => self.handle_variables(req).await,
               _ => Err(anyhow!("Unknown command: {}", req.command)),
           }
       }
   }
   ```
2. LSP server implementation:
   ```rust
   use lsp_server::{Connection, Message};
   use lsp_types::*;
   
   pub struct LspServer {
       runtime: Arc<ScriptRuntime>,
       validator: ScriptValidator,
   }
   
   impl LspServer {
       pub async fn start(&mut self) -> Result<()> {
           let (connection, io_threads) = Connection::stdio();
           
           let capabilities = ServerCapabilities {
               text_document_sync: Some(TextDocumentSyncCapability::Kind(
                   TextDocumentSyncKind::Incremental
               )),
               completion_provider: Some(CompletionOptions::default()),
               hover_provider: Some(HoverProviderCapability::Simple(true)),
               definition_provider: Some(OneOf::Left(true)),
               diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                   DiagnosticOptions::default()
               )),
               ..Default::default()
           };
           
           connection.initialize(serde_json::to_value(capabilities)?)?;
           
           self.main_loop(connection).await?;
           io_threads.join()?;
           
           Ok(())
       }
       
       async fn handle_completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
           let items = self.completion_engine
               .get_completions_at(
                   &params.text_document_position.position,
                   &params.context,
               )
               .into_iter()
               .map(|c| CompletionItem {
                   label: c.text,
                   kind: Some(Self::map_completion_kind(c.kind)),
                   documentation: c.documentation.map(|d| {
                       Documentation::String(d)
                   }),
                   ..Default::default()
               })
               .collect();
           
           Ok(CompletionResponse::Array(items))
       }
   }
   ```
3. VS Code extension stub:
   ```typescript
   // extension.ts
   import * as vscode from 'vscode';
   import { LanguageClient } from 'vscode-languageclient';
   
   export function activate(context: vscode.ExtensionContext) {
       const serverOptions = {
           command: 'llmspell',
           args: ['lsp-server'],
       };
       
       const clientOptions = {
           documentSelector: [{ scheme: 'file', language: 'lua' }],
       };
       
       const client = new LanguageClient(
           'llmspell',
           'LLMSpell Language Server',
           serverOptions,
           clientOptions
       );
       
       client.start();
       
       // Debug adapter
       vscode.debug.registerDebugAdapterDescriptorFactory('llmspell', {
           createDebugAdapterDescriptor: () => {
               return new vscode.DebugAdapterServer(9229);
           }
       });
   }
   ```

**Definition of Done:**
- [ ] DAP server functional
- [ ] LSP server working
- [ ] VS Code connects
- [ ] Protocol compliant
- [ ] Multi-client tested

### Task 9.5.3: Session Recording Foundation
**Priority**: MEDIUM  
**Estimated Time**: 4 hours  
**Assignee**: Integration Team

**Description**: Implement hook-based session recording with foundation for future replay.

**Acceptance Criteria:**
- [ ] Hook events recorded to session log
- [ ] Session metadata captured
- [ ] Recording start/stop commands work
- [ ] Session files properly formatted
- [ ] Foundation for replay established

**Implementation Steps:**
1. Create session recorder:
   ```rust
   pub struct SessionRecorder {
       session_id: Uuid,
       start_time: Instant,
       events: Vec<RecordedEvent>,
       hooks: Arc<HookManager>,
       recording: AtomicBool,
   }
   
   #[derive(Serialize, Deserialize)]
   pub struct RecordedEvent {
       pub timestamp: Duration,
       pub event_type: EventType,
       pub data: serde_json::Value,
   }
   
   #[derive(Serialize, Deserialize)]
   pub enum EventType {
       ScriptStart { script: String },
       HookExecution { name: String, input: Value, output: Value },
       BreakpointHit { file: String, line: usize },
       VariableChange { name: String, old: Value, new: Value },
       ToolInvocation { tool: String, params: Value },
       AgentCall { agent: String, input: Value },
       Error { message: String, stack: Vec<String> },
   }
   ```
2. Hook event recording:
   ```rust
   impl SessionRecorder {
       pub fn start_recording(&self) -> Result<()> {
           self.recording.store(true, Ordering::SeqCst);
           
           // Register recording hook
           self.hooks.register_hook(
               "session_recorder",
               HookType::Universal,
               Box::new(move |ctx: &HookContext| {
                   if self.recording.load(Ordering::SeqCst) {
                       self.record_hook_event(ctx);
                   }
                   HookResult::Continue
               })
           )?;
           
           self.events.push(RecordedEvent {
               timestamp: Duration::ZERO,
               event_type: EventType::ScriptStart {
                   script: self.get_current_script(),
               },
               data: json!({}),
           });
           
           Ok(())
       }
       
       fn record_hook_event(&mut self, ctx: &HookContext) {
           let event = RecordedEvent {
               timestamp: self.start_time.elapsed(),
               event_type: EventType::HookExecution {
                   name: ctx.hook_name.clone(),
                   input: ctx.input.clone(),
                   output: ctx.output.clone(),
               },
               data: json!({
                   "metadata": ctx.metadata,
               }),
           };
           
           self.events.push(event);
       }
   }
   ```
3. Session file format:
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct SessionFile {
       pub version: u32,
       pub session_id: Uuid,
       pub timestamp: DateTime<Utc>,
       pub duration: Duration,
       pub metadata: SessionMetadata,
       pub events: Vec<RecordedEvent>,
   }
   
   #[derive(Serialize, Deserialize)]
   pub struct SessionMetadata {
       pub script_path: PathBuf,
       pub script_hash: String,
       pub environment: HashMap<String, String>,
       pub llmspell_version: String,
       pub platform: String,
   }
   
   impl SessionRecorder {
       pub fn save_session(&self, path: &Path) -> Result<()> {
           let session = SessionFile {
               version: 1,
               session_id: self.session_id,
               timestamp: Utc::now(),
               duration: self.start_time.elapsed(),
               metadata: self.collect_metadata(),
               events: self.events.clone(),
           };
           
           let json = serde_json::to_string_pretty(&session)?;
           std::fs::write(path, json)?;
           
           Ok(())
       }
   }
   ```
4. Replay foundation:
   ```rust
   pub struct SessionReplayer {
       session: SessionFile,
       current_event: usize,
   }
   
   impl SessionReplayer {
       pub fn load(path: &Path) -> Result<Self> {
           let json = std::fs::read_to_string(path)?;
           let session: SessionFile = serde_json::from_str(&json)?;
           
           Ok(Self {
               session,
               current_event: 0,
           })
       }
       
       // Foundation for future replay implementation
       pub fn get_next_event(&mut self) -> Option<&RecordedEvent> {
           if self.current_event < self.session.events.len() {
               let event = &self.session.events[self.current_event];
               self.current_event += 1;
               Some(event)
           } else {
               None
           }
       }
       
       pub fn seek_to_time(&mut self, timestamp: Duration) {
           self.current_event = self.session.events
               .iter()
               .position(|e| e.timestamp >= timestamp)
               .unwrap_or(self.session.events.len());
       }
   }
   ```

**Definition of Done:**
- [ ] Session recording works
- [ ] Events captured correctly
- [ ] Session files valid JSON
- [ ] Replay foundation in place
- [ ] Metadata comprehensive

### Task 9.5.4: Section 9.5 Testing and Code Quality  
**Priority**: CRITICAL  
**Estimated Time**: 5 hours  
**Assignee**: QA Team

**Description**: Comprehensive testing and code quality validation for Advanced Features including hook introspection, protocol implementation, and session recording.

**Acceptance Criteria:**
- [ ] Hook introspection thoroughly tested
- [ ] Debug protocol (DAP/LSP) integration validated
- [ ] Session recording tested end-to-end
- [ ] Protocol compliance verified
- [ ] Zero clippy warnings across all advanced features
- [ ] Code formatted consistently
- [ ] Performance impact of advanced features measured

**Implementation Steps:**
1. Hook introspection tests:
   ```rust
   #[cfg(test)]
   mod hook_introspection_tests {
       use super::*;
       use llmspell_hooks::{HookManager, HookEvent};
       
       #[tokio::test]
       async fn test_hook_inspector_lists_all_hooks() {
           let hook_manager = Arc::new(HookManager::new());
           hook_manager.register_hook("test_hook", |_| async { Ok(()) }).await.unwrap();
           
           let inspector = HookInspector::new(hook_manager.clone());
           let hooks = inspector.list_hooks();
           
           assert_eq!(hooks.len(), 1);
           assert_eq!(hooks[0].name, "test_hook");
       }
       
       #[tokio::test]
       async fn test_hook_execution_tracing() {
           let inspector = create_test_inspector().await;
           
           inspector.start_tracing().await;
           
           // Trigger some hook executions
           let hook_event = HookEvent::new("test_event", serde_json::json!({}));
           inspector.hook_manager.emit_event(hook_event).await.unwrap();
           
           let trace = inspector.get_execution_trace().await;
           assert!(!trace.is_empty());
           assert!(trace.iter().any(|entry| entry.hook_name == "test_hook"));
       }
       
       #[tokio::test]
       async fn test_performance_metrics_collection() {
           let inspector = create_test_inspector().await;
           
           inspector.enable_performance_tracking().await;
           
           // Execute hook that takes some time
           let start = Instant::now();
           inspector.execute_test_hook().await;
           let expected_duration = start.elapsed();
           
           let metrics = inspector.get_performance_metrics().await;
           assert!(metrics.total_execution_time >= expected_duration);
           assert!(metrics.hook_call_count > 0);
       }
   }
   ```

2. Debug protocol tests:
   ```rust
   #[cfg(test)]
   mod debug_protocol_tests {
       use super::*;
       use serde_json::json;
       
       #[tokio::test]
       async fn test_dap_initialization() {
           let mut adapter = DebugAdapter::new();
           
           let init_request = json!({
               "command": "initialize",
               "arguments": {
                   "clientID": "test-client",
                   "adapterID": "llmspell-debug"
               }
           });
           
           let response = adapter.handle_request(init_request).await.unwrap();
           assert_eq!(response["command"], "initialize");
           assert!(response["success"].as_bool().unwrap());
       }
       
       #[tokio::test]
       async fn test_dap_launch_configuration() {
           let mut adapter = DebugAdapter::new();
           adapter.initialize().await.unwrap();
           
           let launch_request = json!({
               "command": "launch",
               "arguments": {
                   "type": "llmspell",
                   "request": "launch",
                   "program": "test_script.lua"
               }
           });
           
           let response = adapter.handle_request(launch_request).await.unwrap();
           assert!(response["success"].as_bool().unwrap());
       }
       
       #[tokio::test]
       async fn test_lsp_completion() {
           let server = LspServer::new().await.unwrap();
           
           let completion_request = json!({
               "jsonrpc": "2.0",
               "id": 1,
               "method": "textDocument/completion",
               "params": {
                   "textDocument": {"uri": "file:///test.lua"},
                   "position": {"line": 0, "character": 10}
               }
           });
           
           let response = server.handle_request(completion_request).await.unwrap();
           assert!(response.get("result").is_some());
           
           let completions = response["result"]["items"].as_array().unwrap();
           assert!(!completions.is_empty());
       }
       
       #[tokio::test]
       async fn test_lsp_diagnostics() {
           let server = LspServer::new().await.unwrap();
           
           let script_with_error = "print('unclosed string";
           let diagnostics = server.validate_script(script_with_error).await.unwrap();
           
           assert!(!diagnostics.is_empty());
           assert_eq!(diagnostics[0].severity, DiagnosticSeverity::ERROR);
           assert!(diagnostics[0].message.contains("unclosed"));
       }
   }
   ```

3. Session recording tests:
   ```rust
   #[cfg(test)]
   mod session_recording_tests {
       use super::*;
       use tempfile::TempDir;
       
       #[tokio::test]
       async fn test_session_recording_creation() {
           let temp_dir = TempDir::new().unwrap();
           let session_path = temp_dir.path().join("test_session.llmspell");
           
           let mut recorder = SessionRecorder::new(&session_path);
           recorder.start_recording().await.unwrap();
           
           // Record some events
           recorder.record_command("print('hello')").await.unwrap();
           recorder.record_output("hello").await.unwrap();
           recorder.record_state_change("test_var", "test_value").await.unwrap();
           
           recorder.stop_recording().await.unwrap();
           
           // Verify session file exists and is valid
           assert!(session_path.exists());
           let session_data = fs::read_to_string(&session_path).unwrap();
           let session: SessionData = serde_json::from_str(&session_data).unwrap();
           
           assert_eq!(session.events.len(), 3);
           assert_eq!(session.events[0].event_type, EventType::Command);
           assert_eq!(session.events[1].event_type, EventType::Output);
           assert_eq!(session.events[2].event_type, EventType::StateChange);
       }
       
       #[tokio::test]
       async fn test_session_metadata_capture() {
           let temp_dir = TempDir::new().unwrap();
           let session_path = temp_dir.path().join("metadata_test.llmspell");
           
           let recorder = SessionRecorder::new(&session_path);
           let metadata = recorder.capture_metadata().await.unwrap();
           
           assert!(!metadata.session_id.is_empty());
           assert!(metadata.start_time > 0);
           assert!(!metadata.llmspell_version.is_empty());
           assert!(!metadata.lua_version.is_empty());
           assert!(!metadata.environment.is_empty());
       }
       
       #[tokio::test]
       async fn test_session_size_limits() {
           let temp_dir = TempDir::new().unwrap();
           let session_path = temp_dir.path().join("size_limit_test.llmspell");
           
           let mut recorder = SessionRecorder::new(&session_path);
           recorder.set_max_size_mb(1); // 1MB limit
           recorder.start_recording().await.unwrap();
           
           // Try to record more than 1MB of data
           for i in 0..10000 {
               let large_output = "x".repeat(1000); // 1KB per iteration
               recorder.record_output(&large_output).await.unwrap();
           }
           
           let session_data = fs::read_to_string(&session_path).unwrap();
           assert!(session_data.len() <= 1024 * 1024 * 2); // Allow some overhead
       }
   }
   ```

4. Protocol compliance tests:
   ```rust
   #[cfg(test)]
   mod protocol_compliance_tests {
       #[tokio::test]
       async fn test_dap_protocol_compliance() {
           let adapter = DebugAdapter::new();
           
           // Test all required DAP methods
           let required_methods = vec![
               "initialize", "launch", "attach", "disconnect",
               "setBreakpoints", "continue", "next", "stepIn", "stepOut",
               "pause", "stackTrace", "scopes", "variables", "evaluate"
           ];
           
           for method in required_methods {
               assert!(adapter.supports_method(method), 
                      "DAP method '{}' not supported", method);
           }
       }
       
       #[tokio::test]
       async fn test_lsp_protocol_compliance() {
           let server = LspServer::new().await.unwrap();
           
           // Test required LSP capabilities
           let capabilities = server.get_capabilities();
           
           assert!(capabilities.text_document_sync.is_some());
           assert!(capabilities.completion_provider.is_some());
           assert!(capabilities.diagnostic_provider.is_some());
           assert!(capabilities.definition_provider == Some(OneOf::Left(true)));
           assert!(capabilities.hover_provider == Some(HoverProviderCapability::Simple(true)));
       }
   }
   ```

5. Performance impact tests:
   ```rust
   #[cfg(test)]
   mod performance_tests {
       #[bench]
       fn bench_hook_introspection_overhead(b: &mut Bencher) {
           let rt = tokio::runtime::Runtime::new().unwrap();
           let inspector = rt.block_on(create_test_inspector());
           
           b.iter(|| {
               rt.block_on(async {
                   let hooks = inspector.list_hooks();
                   black_box(hooks);
               })
           });
       }
       
       #[bench]
       fn bench_session_recording_overhead(b: &mut Bencher) {
           let rt = tokio::runtime::Runtime::new().unwrap();
           let temp_dir = TempDir::new().unwrap();
           let session_path = temp_dir.path().join("bench_session.llmspell");
           let mut recorder = SessionRecorder::new(&session_path);
           
           rt.block_on(recorder.start_recording()).unwrap();
           
           b.iter(|| {
               rt.block_on(async {
                   recorder.record_command("print('benchmark')").await.unwrap();
               })
           });
       }
       
       #[bench]
       fn bench_dap_message_handling(b: &mut Bencher) {
           let rt = tokio::runtime::Runtime::new().unwrap();
           let mut adapter = DebugAdapter::new();
           rt.block_on(adapter.initialize()).unwrap();
           
           let test_request = json!({
               "command": "stackTrace",
               "arguments": {"threadId": 1}
           });
           
           b.iter(|| {
               rt.block_on(async {
                   let response = adapter.handle_request(test_request.clone()).await.unwrap();
                   black_box(response);
               })
           });
       }
   }
   ```

6. Code quality validation:
   ```bash
   # Zero clippy warnings
   cargo clippy --all-features --all-targets -- -D warnings
   
   # Consistent formatting  
   cargo fmt --all -- --check
   
   # Protocol compliance validation
   cargo test --features protocol-compliance -- --test-threads=1
   
   # Performance regression tests
   cargo bench --features benchmarks
   
   # Memory usage validation
   valgrind --tool=massif ./target/debug/llmspell run test_advanced_features.lua
   ```

**Definition of Done:**
- [ ] Hook introspection tests cover all functionality
- [ ] DAP/LSP protocol compliance verified
- [ ] Session recording robust and size-limited
- [ ] Protocol performance within acceptable limits
- [ ] Zero clippy warnings in all advanced features
- [ ] All code formatted consistently
- [ ] Memory usage stable under load
- [ ] Integration with Phase 4 hooks validated

---

## Phase 9.6: Testing and Documentation (Day 15)

### Task 9.6.1: Comprehensive Test Suite and Final Validation
**Priority**: CRITICAL  
**Estimated Time**: 8 hours  
**Assignee**: QA Team

**Description**: Build comprehensive test coverage for all REPL and debug features with final Phase 9 validation and integration tests.

**Acceptance Criteria:**
- [ ] Complete unit test coverage for all Phase 9 components
- [ ] End-to-end integration tests covering full debugging workflows
- [ ] Cross-component integration validated
- [ ] Performance benchmarks establish baseline
- [ ] >90% code coverage achieved across all Phase 9 code
- [ ] Zero clippy warnings in entire codebase
- [ ] All code formatted consistently
- [ ] Stress testing validates stability
- [ ] Memory leak detection passes

**Implementation Steps:**
1. Complete unit test coverage:
   ```rust
   #[cfg(test)]
   mod comprehensive_repl_tests {
       use super::*;
       use llmspell_testing::*;
       
       #[tokio::test]
       async fn test_repl_full_lifecycle() {
           let session = ReplSession::new_with_config(TestConfig::default()).await.unwrap();
           
           // Test session creation
           assert!(session.is_active());
           assert_eq!(session.get_prompt(), "> ");
           
           // Test basic command execution
           let result = session.execute_command("print('hello world')").await.unwrap();
           assert_eq!(result.output, "hello world\n");
           assert!(result.success);
           
           // Test error handling
           let error_result = session.execute_command("invalid_syntax(").await.unwrap();
           assert!(!error_result.success);
           assert!(error_result.error.is_some());
           
           // Test state persistence
           session.execute_command("test_var = 42").await.unwrap();
           let state_result = session.execute_command("return test_var").await.unwrap();
           assert!(state_result.output.contains("42"));
           
           // Test session shutdown
           session.shutdown().await.unwrap();
           assert!(!session.is_active());
       }
       
       #[tokio::test]
       async fn test_repl_command_history() {
           let session = create_test_repl_session().await;
           
           // Execute several commands
           session.execute_command("print(1)").await.unwrap();
           session.execute_command("print(2)").await.unwrap();
           session.execute_command("print(3)").await.unwrap();
           
           let history = session.get_command_history();
           assert_eq!(history.len(), 3);
           assert_eq!(history[0], "print(1)");
           assert_eq!(history[1], "print(2)");
           assert_eq!(history[2], "print(3)");
           
           // Test history navigation
           session.navigate_history(-1).await; // Up to "print(3)"
           assert_eq!(session.get_current_command(), "print(3)");
           
           session.navigate_history(-1).await; // Up to "print(2)"
           assert_eq!(session.get_current_command(), "print(2)");
           
           session.navigate_history(1).await; // Down to "print(3)"
           assert_eq!(session.get_current_command(), "print(3)");
       }
       
       #[tokio::test]
       async fn test_repl_tab_completion() {
           let session = create_test_repl_session().await;
           
           // Test API completion
           let completions = session.get_completions("Tool.in", 7).await.unwrap();
           assert!(completions.contains(&"invoke".to_string()));
           
           // Test variable completion
           session.execute_command("test_variable = 'hello'").await.unwrap();
           let var_completions = session.get_completions("test_v", 6).await.unwrap();
           assert!(var_completions.contains(&"test_variable".to_string()));
           
           // Test function completion
           let func_completions = session.get_completions("pri", 3).await.unwrap();
           assert!(func_completions.contains(&"print".to_string()));
       }
   }
   
   #[cfg(test)]
   mod comprehensive_debug_tests {
       #[tokio::test]
       async fn test_enhanced_error_reporting_full() {
           let session = create_test_repl_session().await;
           
           // Test syntax error enhancement
           let result = session.execute_command("function test() return end").await.unwrap();
           assert!(!result.success);
           
           let enhanced_error = result.enhanced_error.unwrap();
           assert!(enhanced_error.suggestions.len() > 0);
           assert!(enhanced_error.location.is_some());
           assert!(enhanced_error.context.is_some());
           assert!(enhanced_error.rust_style_format.contains("error[E"));
           
           // Test runtime error enhancement
           let runtime_result = session.execute_command("error('test error')").await.unwrap();
           let runtime_error = runtime_result.enhanced_error.unwrap();
           assert!(runtime_error.stack_trace.len() > 0);
           assert!(runtime_error.suggestions.contains(&"Check the error condition".to_string()));
       }
       
       #[tokio::test]
       async fn test_breakpoint_functionality_comprehensive() {
           let debugger = Debugger::new_with_session(create_test_repl_session().await);
           
           // Test setting breakpoints
           let script = r#"
               function fibonacci(n)
                   if n <= 1 then
                       return n
                   else
                       return fibonacci(n-1) + fibonacci(n-2)
                   end
               end
               
               result = fibonacci(10)
           "#;
           
           // Set breakpoint at line 4 (return n)
           debugger.set_breakpoint("test_script", 4).await.unwrap();
           
           // Execute and hit breakpoint
           let exec_result = debugger.execute_with_debugging(script).await.unwrap();
           assert!(exec_result.hit_breakpoint);
           assert_eq!(exec_result.current_line, 4);
           
           // Test variable inspection at breakpoint
           let variables = debugger.get_local_variables().await.unwrap();
           assert!(variables.contains_key("n"));
           
           // Test step over
           debugger.step_over().await.unwrap();
           let step_result = debugger.get_current_state().await.unwrap();
           assert_ne!(step_result.current_line, 4); // Should have moved
           
           // Test continue
           debugger.continue_execution().await.unwrap();
           let final_result = debugger.wait_for_completion().await.unwrap();
           assert!(final_result.completed);
       }
       
       #[tokio::test]
       async fn test_variable_inspection_deep() {
           let debugger = create_test_debugger().await;
           
           let script = r#"
               local complex_table = {
                   numbers = {1, 2, 3, 4, 5},
                   nested = {
                       deep = {
                           value = "found it"
                       }
                   },
                   func = function(x) return x * 2 end
               }
           "#;
           
           debugger.set_breakpoint("test", 10).await.unwrap();
           debugger.execute_with_debugging(script).await.unwrap();
           
           // Test shallow inspection
           let vars = debugger.get_local_variables().await.unwrap();
           assert!(vars.contains_key("complex_table"));
           
           // Test deep inspection
           let table_contents = debugger.inspect_variable_deep("complex_table").await.unwrap();
           assert_eq!(table_contents.get("numbers.1").unwrap(), &"1".to_string());
           assert_eq!(table_contents.get("nested.deep.value").unwrap(), &"found it".to_string());
           
           // Test lazy expansion
           let lazy_result = debugger.expand_variable_lazy("complex_table", 2).await.unwrap();
           assert!(lazy_result.children.len() <= 2); // Should limit expansion
       }
   }
   ```

2. End-to-end integration tests:
   ```rust
   #[cfg(test)]
   mod integration_tests {
       #[tokio::test]
       async fn test_complete_debugging_workflow() {
           // Create a complete debugging session
           let temp_dir = create_temp_project().await;
           let script_path = temp_dir.join("debug_test.lua");
           
           fs::write(&script_path, r#"
               function process_data(items)
                   local results = {}
                   for i, item in ipairs(items) do
                       if item > 0 then
                           table.insert(results, item * 2)
                       end
                   end
                   return results
               end
               
               local data = {1, -2, 3, 0, 5}
               local processed = process_data(data)
               print("Processed:", table.concat(processed, ", "))
           "#).unwrap();
           
           // Start REPL with debugging enabled
           let session = ReplSession::new_with_debugging(&temp_dir).await.unwrap();
           
           // Enable hot reload and set breakpoint
           session.execute_command(".reload auto").await.unwrap();
           session.execute_command(".breakpoint debug_test.lua:4").await.unwrap();
           
           // Run script and hit breakpoint
           let result = session.execute_command("dofile('debug_test.lua')").await.unwrap();
           assert!(result.debug_session.is_some());
           
           let debug_session = result.debug_session.unwrap();
           
           // Inspect variables at breakpoint
           let locals = debug_session.get_local_variables().await.unwrap();
           assert!(locals.contains_key("item"));
           assert!(locals.contains_key("i"));
           assert!(locals.contains_key("results"));
           
           // Step through and continue
           debug_session.step_over().await.unwrap();
           debug_session.continue_execution().await.unwrap();
           
           // Verify final output
           let final_result = debug_session.wait_for_completion().await.unwrap();
           assert!(final_result.output.contains("Processed: 2, 6, 10"));
           
           // Test hot reload
           fs::write(&script_path, r#"
               function process_data(items)
                   local results = {}
                   for i, item in ipairs(items) do
                       if item > 0 then
                           table.insert(results, item * 3)  -- Changed from *2 to *3
                       end
                   end
                   return results
               end
               
               local data = {1, -2, 3, 0, 5}
               local processed = process_data(data)
               print("Processed:", table.concat(processed, ", "))
           "#).unwrap();
           
           // Wait for hot reload
           tokio::time::sleep(Duration::from_millis(100)).await;
           
           // Run again and verify change
           let reload_result = session.execute_command("dofile('debug_test.lua')").await.unwrap();
           assert!(reload_result.output.contains("Processed: 3, 9, 15"));
       }
       
       #[tokio::test]
       async fn test_cross_component_integration() {
           let session = ReplSession::new_with_all_features().await.unwrap();
           
           // Test hook introspection + debugging
           session.execute_command(".hooks enable debug_hook").await.unwrap();
           session.execute_command(".hooks trace").await.unwrap();
           
           // Execute something that triggers hooks
           let result = session.execute_command("Tool.invoke('calculator', {operation = 'add', a = 5, b = 3})").await.unwrap();
           
           // Verify hook execution was traced
           let trace = session.get_hook_trace().await.unwrap();
           assert!(!trace.is_empty());
           
           // Test profiling + debugging integration
           session.execute_command(".profile start").await.unwrap();
           session.execute_command(".breakpoint set current:1").await.unwrap();
           
           let profile_result = session.execute_command("
               for i = 1, 1000 do
                   local x = i * i
               end
           ").await.unwrap();
           
           session.execute_command(".profile stop").await.unwrap();
           
           let profile_report = session.get_profile_report().await.unwrap();
           assert!(profile_report.total_time > Duration::from_millis(0));
           assert!(!profile_report.function_times.is_empty());
       }
       
       #[tokio::test]
       async fn test_session_recording_integration() {
           let temp_dir = create_temp_project().await;
           let session_file = temp_dir.join("debug_session.llmspell");
           
           let session = ReplSession::new_with_recording(&session_file).await.unwrap();
           
           // Execute a complete debugging workflow
           session.execute_command(".record start").await.unwrap();
           session.execute_command("function test(x) return x * 2 end").await.unwrap();
           session.execute_command(".breakpoint set test:1").await.unwrap();
           session.execute_command("result = test(21)").await.unwrap();
           session.execute_command(".vars").await.unwrap();
           session.execute_command(".continue").await.unwrap();
           session.execute_command(".record stop").await.unwrap();
           
           // Verify session file was created and contains expected events
           assert!(session_file.exists());
           
           let session_data = fs::read_to_string(&session_file).unwrap();
           let parsed_session: SessionData = serde_json::from_str(&session_data).unwrap();
           
           // Should have events for all commands
           assert!(parsed_session.events.len() >= 7);
           
           // Verify event types are correct
           let command_events: Vec<_> = parsed_session.events
               .iter()
               .filter(|e| e.event_type == EventType::Command)
               .collect();
           assert!(command_events.len() >= 6);
           
           // Verify breakpoint event was recorded
           let breakpoint_events: Vec<_> = parsed_session.events
               .iter()
               .filter(|e| e.event_type == EventType::Breakpoint)
               .collect();
           assert!(!breakpoint_events.is_empty());
       }
   }
   ```

3. Performance and stress tests:
   ```rust
   #[cfg(test)]
   mod performance_stress_tests {
       #[tokio::test]
       async fn test_repl_performance_under_load() {
           let session = create_test_repl_session().await;
           let start_time = Instant::now();
           
           // Execute 1000 commands rapidly
           for i in 0..1000 {
               let command = format!("local x{} = {}", i, i);
               session.execute_command(&command).await.unwrap();
           }
           
           let total_time = start_time.elapsed();
           let avg_time_per_command = total_time / 1000;
           
           // Each command should take less than 5ms on average
           assert!(avg_time_per_command < Duration::from_millis(5));
       }
       
       #[tokio::test]
       async fn test_debugger_memory_stability() {
           let session = ReplSession::new_with_debugging().await.unwrap();
           
           let initial_memory = get_process_memory_usage();
           
           // Create and destroy many debugging sessions
           for _ in 0..100 {
               session.execute_command(".breakpoint set current:1").await.unwrap();
               session.execute_command("for i=1,100 do local x = i end").await.unwrap();
               session.execute_command(".breakpoint clear all").await.unwrap();
           }
           
           // Force garbage collection
           session.execute_command("collectgarbage('collect')").await.unwrap();
           tokio::time::sleep(Duration::from_millis(100)).await;
           
           let final_memory = get_process_memory_usage();
           let memory_growth = final_memory - initial_memory;
           
           // Memory growth should be less than 10MB
           assert!(memory_growth < 10 * 1024 * 1024);
       }
       
       #[bench]
       fn bench_error_enhancement_overhead(b: &mut Bencher) {
           let rt = tokio::runtime::Runtime::new().unwrap();
           let session = rt.block_on(create_test_repl_session());
           
           b.iter(|| {
               rt.block_on(async {
                   let result = session.execute_command("error('test error')").await.unwrap();
                   black_box(result.enhanced_error);
               })
           });
       }
       
       #[bench]
       fn bench_breakpoint_checking_overhead(b: &mut Bencher) {
           let rt = tokio::runtime::Runtime::new().unwrap();
           let debugger = rt.block_on(create_test_debugger());
           
           rt.block_on(debugger.set_breakpoint("test", 1)).unwrap();
           
           b.iter(|| {
               rt.block_on(async {
                   let has_bp = debugger.check_breakpoint("test", 1).await;
                   black_box(has_bp);
               })
           });
       }
   }
   ```

4. Code quality validation:
   ```bash
   # Comprehensive clippy check
   cargo clippy --workspace --all-features --all-targets -- -D warnings
   
   # Formatting validation
   cargo fmt --all -- --check
   
   # Documentation coverage
   RUSTDOCFLAGS="-D missing_docs" cargo doc --workspace --all-features --no-deps
   
   # Test all features
   cargo test --workspace --all-features
   
   # Run benchmarks
   cargo bench --all-features
   
   # Memory leak detection
   valgrind --tool=memcheck --leak-check=full ./target/debug/llmspell repl
   
   # Address sanitizer
   RUSTFLAGS="-Z sanitizer=address" cargo test --target x86_64-unknown-linux-gnu
   ```

5. Coverage analysis:
   ```bash
   # Install coverage tools
   cargo install cargo-tarpaulin
   
   # Generate coverage report
   cargo tarpaulin --workspace --all-features --out Html --output-dir coverage/
   
   # Verify >90% coverage
   cargo tarpaulin --workspace --all-features --fail-under 90
   ```

**Definition of Done:**
- [ ] All unit tests pass with >90% coverage
- [ ] End-to-end integration tests validate complete workflows
- [ ] Cross-component integration verified
- [ ] Performance benchmarks establish acceptable baselines
- [ ] Stress tests prove stability under load
- [ ] Zero clippy warnings across entire codebase
- [ ] All code formatted consistently
- [ ] Memory leak tests pass
- [ ] Documentation coverage at 100%
- [ ] CI pipeline validates all checks

### Task 9.6.2: Documentation and Tutorials
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: Documentation Team

**Description**: Create comprehensive documentation and tutorials for REPL and debugging.

**Acceptance Criteria:**
- [ ] REPL user guide complete
- [ ] Debug feature documentation
- [ ] Interactive tutorials created
- [ ] API reference updated
- [ ] Troubleshooting guide written

**Implementation Steps:**
1. Write REPL user guide
2. Document debug features
3. Create interactive tutorials
4. Update API docs
5. Add troubleshooting section

**Definition of Done:**
- [ ] Documentation complete
- [ ] Examples working
- [ ] Tutorials tested
- [ ] Published to docs site

---

## Success Metrics

### Performance Targets
- REPL startup time: <100ms
- Command response time: <50ms
- Tab completion: <100ms
- Error enhancement: <10ms overhead
- Breakpoint checking: <1ms
- Hot reload: <500ms
- Memory overhead: <50MB

### Quality Metrics
- Test coverage: >90%
- Documentation coverage: 100%
- Zero critical bugs
- <5ms debug overhead in production

### User Experience Metrics
- 80% reduction in debug time
- 90% of errors show actionable suggestions
- 95% of users can debug without documentation

---

## Risk Mitigation

### Technical Risks
1. **mlua debugging limitations**: Some features may require mlua patches
   - Mitigation: Contribute patches upstream, maintain fork if needed

2. **Performance overhead**: Debugging may slow execution
   - Mitigation: Lazy evaluation, conditional compilation

3. **Protocol compatibility**: DAP/LSP specs may change
   - Mitigation: Version pinning, compatibility layer

### Operational Risks
1. **Complexity**: Many moving parts to integrate
   - Mitigation: Incremental development, thorough testing

2. **User adoption**: Learning curve for new features
   - Mitigation: Excellent documentation, progressive disclosure

---

## Dependencies

### External Crates
- `inquire = "0.6"` - Interactive prompts
- `crossterm = "0.27"` - Terminal control
- `notify = "6.0"` - File watching
- `syntect = "5.0"` - Syntax highlighting
- `flamegraph = "0.6"` - Profiling visualization
- `dap = "0.4"` - Debug Adapter Protocol
- `lsp-server = "0.7"` - Language Server Protocol
- `fuzzy-matcher = "0.3"` - Fuzzy completion

### Internal Dependencies
- Phase 4: Hook system for introspection
- Phase 5: State management for persistence
- Phase 8: Vector storage for search

---

## Notes

- REPL is the primary interface, debugging is integrated naturally
- Focus on developer experience and productivity
- Performance critical for interactive use
- Extensibility important for future enhancements
- This phase transforms llmspell from powerful to delightful