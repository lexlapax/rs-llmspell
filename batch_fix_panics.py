import os

fixes = [
    ("composition/tool_composition.rs", 315, "    /// Execute the composition with the given tool provider\n    pub async fn execute<T>(", "    /// Execute the composition with the given tool provider\n    ///\n    /// # Panics\n    ///\n    /// Panics if a RwLock is poisoned\n    pub async fn execute<T>("),
    ("di.rs", 193, "    /// Add a tool to the container\n    #[must_use]\n    pub fn with_tool(", "    /// Add a tool to the container\n    ///\n    /// # Panics\n    ///\n    /// Panics if the RwLock is poisoned\n    #[must_use]\n    pub fn with_tool("),
    ("lifecycle/benchmarks.rs", 27, "    #[must_use]\n    pub fn duration(&self) -> Duration {", "    /// Get benchmark duration\n    ///\n    /// # Panics\n    ///\n    /// Panics if the RwLock is poisoned\n    #[must_use]\n    pub fn duration(&self) -> Duration {"),
    ("lifecycle/benchmarks.rs", 79, "    #[must_use]\n    pub fn percentile(&self, p: f64) -> Duration {", "    /// Get percentile duration\n    ///\n    /// # Panics\n    ///\n    /// Panics if the RwLock is poisoned\n    #[must_use]\n    pub fn percentile(&self, p: f64) -> Duration {"),
    ("lifecycle/shutdown.rs", 211, "    /// Get remaining agents count\n    #[must_use]\n    pub fn remaining_agents(&self) -> usize {", "    /// Get remaining agents count\n    ///\n    /// # Panics\n    ///\n    /// Panics if the RwLock is poisoned\n    #[must_use]\n    pub fn remaining_agents(&self) -> usize {"),
    ("monitoring/events.rs", 413, "    /// Export events to a specific format\n    #[must_use]\n    pub fn export(&self, format: ExportFormat) -> String {", "    /// Export events to a specific format\n    ///\n    /// # Panics\n    ///\n    /// Panics if the RwLock is poisoned\n    #[must_use]\n    pub fn export(&self, format: ExportFormat) -> String {"),
    ("monitoring/performance.rs", 96, "    /// Record an operation\n    pub fn record_operation(&self, operation: String, duration: Duration) {", "    /// Record an operation\n    ///\n    /// # Panics\n    ///\n    /// Panics if the Mutex is poisoned\n    pub fn record_operation(&self, operation: String, duration: Duration) {"),
    ("monitoring/performance.rs", 393, "    /// Get performance metrics\n    #[must_use]\n    pub fn get_metrics(&self) -> PerformanceMetrics {", "    /// Get performance metrics\n    ///\n    /// # Panics\n    ///\n    /// Panics if the Mutex is poisoned\n    #[must_use]\n    pub fn get_metrics(&self) -> PerformanceMetrics {"),
    ("registry/discovery.rs", 242, "    /// Search for agents by capabilities\n    #[must_use]\n    pub fn search_by_capabilities(&self, capabilities: &[String]) -> Vec<AgentInfo> {", "    /// Search for agents by capabilities\n    ///\n    /// # Panics\n    ///\n    /// Panics if the RwLock is poisoned\n    #[must_use]\n    pub fn search_by_capabilities(&self, capabilities: &[String]) -> Vec<AgentInfo> {"),
    ("templates/mod.rs", 49, "    /// Get all registered template names\n    #[must_use]\n    pub fn list_templates(&self) -> Vec<String> {", "    /// Get all registered template names\n    ///\n    /// # Panics\n    ///\n    /// Panics if the RwLock is poisoned\n    #[must_use]\n    pub fn list_templates(&self) -> Vec<String> {"),
    ("testing/mocks.rs", 164, "    /// Get execution count\n    ///\n    /// # Panics\n    ///\n    /// Panics if the Mutex is poisoned\n    #[must_use]\n    pub fn execution_count(&self) -> usize {", "    /// Get execution count\n    ///\n    /// # Panics\n    ///\n    /// Panics if the Mutex is poisoned\n    #[must_use]\n    pub fn execution_count(&self) -> usize {"),
]

for filepath, line_num, old_str, new_str in fixes:
    full_path = f"/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-agents/src/{filepath}"
    print(f"Processing {filepath}...")
    
    with open(full_path, 'r') as f:
        content = f.read()
    
    if old_str in content:
        content = content.replace(old_str, new_str, 1)
        with open(full_path, 'w') as f:
            f.write(content)
        print(f"  Fixed line {line_num}")
    else:
        print(f"  WARNING: Could not find text to replace at line {line_num}")

print("Done\!")
