// ABOUTME: Configuration for test runner behavior
// ABOUTME: Controls test execution parameters like parallelism and output

#[derive(Debug, Clone, Default)]
pub struct TestRunnerConfig {
    pub verbose: bool,
    pub release: bool,
    pub jobs: usize, // 0 means use number of CPUs
    pub nocapture: bool,
}
