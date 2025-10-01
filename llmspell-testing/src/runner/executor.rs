// ABOUTME: Test execution engine that runs tests and benchmarks
// ABOUTME: Handles cargo invocation, output formatting, and result aggregation

use super::{TestCategory, TestRunnerConfig};
use llmspell_utils::terminal::Colorize;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Instant;

pub struct TestRunner {
    config: TestRunnerConfig,
}

impl TestRunner {
    pub fn new(config: TestRunnerConfig) -> Self {
        Self { config }
    }

    pub fn run_tests(
        &self,
        categories: Vec<String>,
        filter: Option<String>,
        coverage: bool,
        format: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let categories: Result<Vec<_>, _> = categories
            .iter()
            .map(|c| c.parse::<TestCategory>())
            .collect();
        let categories = categories?;

        // If no categories specified, show help
        if categories.is_empty() {
            println!("{}", "No test categories specified!".red());
            println!();
            println!("Usage: llmspell-test run <CATEGORY>...");
            println!(
                "Available categories: unit, integration, agent, scenario, lua, performance, all"
            );
            return Ok(());
        }

        println!(
            "{} {}",
            "Running tests for:".green().bold(),
            categories
                .iter()
                .map(|c| c.name())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!();

        let start = Instant::now();
        let mut total_passed = 0;
        let mut total_failed = 0;
        let mut failed_categories = Vec::new();

        // Expand "all" category
        let categories_to_run = if categories.contains(&TestCategory::All) {
            TestCategory::all_categories()
        } else {
            categories
        };

        // Run each category
        for category in &categories_to_run {
            if category == &TestCategory::Performance {
                // Skip performance tests in regular test runs
                println!(
                    "{}: {} (use 'llmspell-test bench' instead)",
                    "Skipping".yellow(),
                    category.name()
                );
                continue;
            }

            println!("{} {} tests...", "Running".cyan(), category.name());

            match self.run_category_tests(category, &filter, &format) {
                Ok((passed, failed)) => {
                    total_passed += passed;
                    total_failed += failed;
                    if failed > 0 {
                        failed_categories.push(category.name());
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{}: Failed to run {} tests: {}",
                        "Error".red().bold(),
                        category.name(),
                        e
                    );
                    failed_categories.push(category.name());
                    total_failed += 1;
                }
            }
            println!();
        }

        // Summary
        let duration = start.elapsed();
        println!("{}", "=".repeat(60));
        println!("{}", "Test Summary".green().bold());
        println!("{}: {:?}", "Duration".cyan(), duration);
        println!("{}: {}", "Passed".green(), total_passed);
        println!("{}: {}", "Failed".red(), total_failed);

        if !failed_categories.is_empty() {
            println!();
            println!(
                "{}: {}",
                "Failed categories".red().bold(),
                failed_categories.join(", ")
            );
            return Err("Some tests failed".into());
        }

        println!();
        println!("{}", "All tests passed! 🎉".green().bold());

        // Run coverage if requested
        if coverage {
            println!();
            println!("{}", "Generating coverage report...".cyan());
            self.run_coverage(&categories_to_run)?;
        }

        Ok(())
    }

    fn run_category_tests(
        &self,
        category: &TestCategory,
        filter: &Option<String>,
        format: &str,
    ) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let mut cmd = Command::new("cargo");
        cmd.arg("test");
        cmd.arg("-p").arg("llmspell-testing");
        cmd.arg("--features").arg(category.feature_name());

        if self.config.release {
            cmd.arg("--release");
        }

        if self.config.jobs > 0 {
            cmd.arg("-j").arg(self.config.jobs.to_string());
        }

        if let Some(filter) = filter {
            cmd.arg(filter);
        }

        if self.config.nocapture {
            cmd.arg("--").arg("--nocapture");
        } else if !self.config.verbose {
            cmd.arg("--quiet");
        }

        // Execute and capture output
        let output = if self.config.verbose || format == "pretty" {
            // Stream output in real-time
            let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

            let stdout = child.stdout.take().unwrap();
            let reader = BufReader::new(stdout);

            let mut passed = 0;
            let mut failed = 0;

            for line in reader.lines() {
                let line = line?;
                // Parse test results
                if line.contains("test result:") {
                    if let Some(captures) = parse_test_results(&line) {
                        passed = captures.0;
                        failed = captures.1;
                    }
                }
                if self.config.verbose {
                    println!("{}", line);
                }
            }

            let status = child.wait()?;
            if !status.success() && failed == 0 {
                // Compilation or other error
                failed = 1;
            }

            Ok((passed, failed))
        } else {
            // Quiet mode - just get the results
            let output = cmd.output()?;

            if output.status.success() {
                // Try to parse test count from output
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some((passed, failed)) = extract_test_counts(&stdout) {
                    Ok((passed, failed))
                } else {
                    // Assume all passed if we can't parse
                    Ok((1, 0))
                }
            } else {
                Ok((0, 1))
            }
        };

        output
    }

    pub fn run_benchmarks(
        &self,
        filter: Option<String>,
        save: Option<String>,
        baseline: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "Running benchmarks...".green().bold());
        println!();

        let mut cmd = Command::new("cargo");
        cmd.arg("bench");
        cmd.arg("-p").arg("llmspell-testing");

        if self.config.release {
            cmd.arg("--release");
        }

        if let Some(filter) = filter {
            cmd.arg("--bench").arg(filter);
        }

        // Criterion-specific options
        let mut bench_args = Vec::new();

        if let Some(ref save_name) = save {
            bench_args.push(format!("--save-baseline={}", save_name));
        }

        if let Some(baseline) = baseline {
            bench_args.push(format!("--baseline={}", baseline));
        }

        if !bench_args.is_empty() {
            cmd.arg("--");
            for arg in bench_args {
                cmd.arg(arg);
            }
        }

        // Execute with streaming output
        let mut child = cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let status = child.wait()?;

        if !status.success() {
            return Err("Benchmark execution failed".into());
        }

        println!();
        println!("{}", "Benchmarks completed successfully!".green().bold());

        if save.is_some() {
            println!("Baseline saved for future comparisons");
        }

        Ok(())
    }

    fn run_coverage(&self, categories: &[TestCategory]) -> Result<(), Box<dyn std::error::Error>> {
        // Delegate to the test-coverage.sh script
        let mut cmd = Command::new("./scripts/test-coverage.sh");

        // Determine coverage type based on categories
        let coverage_type = if categories.len() == TestCategory::all_categories().len() {
            "all"
        } else if categories.len() == 1 {
            match categories[0] {
                TestCategory::Unit => "unit",
                TestCategory::Integration => "integration",
                _ => "all",
            }
        } else {
            "all"
        };

        cmd.arg(coverage_type);
        cmd.arg("html"); // Default to HTML output

        let status = cmd.status()?;

        if !status.success() {
            return Err("Coverage generation failed".into());
        }

        Ok(())
    }
}

fn parse_test_results(line: &str) -> Option<(usize, usize)> {
    // Parse lines like: "test result: ok. 42 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
    let parts: Vec<&str> = line.split(';').collect();

    let mut passed = 0;
    let mut failed = 0;

    for part in parts {
        let part = part.trim();
        if part.contains("passed") {
            if let Some(num) = part.split_whitespace().next() {
                passed = num.parse().unwrap_or(0);
            }
        } else if part.contains("failed") {
            if let Some(num) = part.split_whitespace().next() {
                failed = num.parse().unwrap_or(0);
            }
        }
    }

    Some((passed, failed))
}

fn extract_test_counts(output: &str) -> Option<(usize, usize)> {
    for line in output.lines() {
        if line.contains("test result:") {
            return parse_test_results(line);
        }
    }
    None
}
