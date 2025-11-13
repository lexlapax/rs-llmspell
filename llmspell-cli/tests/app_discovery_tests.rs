//! ABOUTME: Baseline tests for Task 10.17.1.1 - Document current state before changes
//! ABOUTME: Tests embedded resources size and filesystem structure

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Baseline: verify filesystem apps have required structure
#[test]
fn test_filesystem_apps_have_required_structure() {
    let current_dir = std::env::current_dir().unwrap();
    println!("Test running from: {:?}", current_dir);

    let apps_dir = PathBuf::from("../examples/script-users/applications");
    println!(
        "Looking for apps at: {:?}",
        apps_dir.canonicalize().unwrap_or(apps_dir.clone())
    );
    assert!(apps_dir.exists(), "Applications directory should exist");

    // Test core apps that exist in both embedded and filesystem
    let expected_apps = vec!["file-organizer", "content-creator", "webapp-creator"];

    for app_name in expected_apps {
        let app_dir = apps_dir.join(app_name);
        assert!(app_dir.exists(), "App {} directory should exist", app_name);

        let main_lua = app_dir.join("main.lua");
        assert!(main_lua.exists(), "App {} should have main.lua", app_name);

        let config_toml = app_dir.join("config.toml");
        assert!(
            config_toml.exists(),
            "App {} should have config.toml",
            app_name
        );

        // Verify files are not empty
        let lua_content = fs::read_to_string(&main_lua).unwrap();
        assert!(
            !lua_content.trim().is_empty(),
            "main.lua should not be empty for {}",
            app_name
        );

        let config_content = fs::read_to_string(&config_toml).unwrap();
        assert!(
            !config_content.trim().is_empty(),
            "config.toml should not be empty for {}",
            app_name
        );
    }

    println!("BASELINE: Filesystem apps have proper structure");
}

/// Baseline: count all filesystem applications
#[test]
fn test_count_filesystem_applications() {
    let apps_dir = PathBuf::from("../examples/script-users/applications");
    assert!(apps_dir.exists(), "Applications directory should exist");

    let entries = fs::read_dir(&apps_dir).unwrap();
    let mut app_count = 0;
    let mut app_names = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            let app_name = entry.file_name().to_string_lossy().to_string();
            let main_lua = entry.path().join("main.lua");
            if main_lua.exists() {
                app_count += 1;
                app_names.push(app_name);
            }
        }
    }

    println!(
        "BASELINE: Found {} filesystem applications: {:?}",
        app_count, app_names
    );

    // Should find around 10-11 apps
    assert!(app_count >= 8, "Expected at least 8 filesystem apps");
    assert!(app_count <= 15, "Expected at most 15 filesystem apps");
}

/// Baseline: document binary size impact
#[test]
fn test_document_binary_size() {
    let binary_path = PathBuf::from("../target/release/llmspell");

    if binary_path.exists() {
        let metadata = fs::metadata(&binary_path).unwrap();
        let size_bytes = metadata.len();
        let size_mb = size_bytes as f64 / 1024.0 / 1024.0;

        println!(
            "BASELINE: Current release binary size: {} bytes ({:.1} MB)",
            size_bytes, size_mb
        );

        // Document for later comparison
        // Phase 11: ~21MB (base system)
        // Phase 12: ~35MB (templates + multi-agent)
        // Phase 13: ~66MB (memory + graph + context + local LLM + workflow-template delegation)
        //   - Candle ML framework with Metal GPU support (~15MB)
        //   - Tokenizers with embedded models (~10MB)
        //   - ZeroMQ native bindings (~3MB)
        //   - Workflow-template delegation (Task 13.13) (~3MB)
        // Threshold updated after Phase 13.13 completion (workflow-template delegation)
        assert!(size_bytes > 15_000_000, "Expected binary > 15MB");
        assert!(size_bytes < 75_000_000, "Expected binary < 75MB");
    } else {
        println!("BASELINE: Release binary not found - cannot measure current size");
    }
}

/// Test app command with nonexistent app to document error handling
#[test]
fn test_app_nonexistent_app() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("llmspell"));
    cmd.arg("app")
        .arg("run")
        .arg("nonexistent-app")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Application 'nonexistent-app' not found",
        ));

    println!("BASELINE: App command properly handles nonexistent apps");
}

/// Document that current app command only shows preview (broken execution)
#[test]
fn test_document_app_command_current_behavior() {
    // This test documents the current broken state where app command only shows preview
    // The app command should execute the app but currently just shows script content

    // We can't test the full execution here because of path issues, but we document the
    // expected behavior for sub-task 10.17.1.3

    println!("BASELINE: App command currently shows preview instead of executing");
    println!("BASELINE: Sub-task 10.17.1.3 will fix execution to actually run apps");

    // This test always passes - it's just documentation
}

/// Test performance baseline of app binary startup
#[test]
fn test_binary_startup_performance() {
    use std::time::Instant;

    let start = Instant::now();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("llmspell"));
    cmd.arg("--help").assert().success();

    let duration = start.elapsed();
    println!("BASELINE: Binary startup took: {:?}", duration);

    // Should be reasonably fast
    assert!(
        duration.as_secs() < 3,
        "Binary startup should be < 3 seconds"
    );
}
