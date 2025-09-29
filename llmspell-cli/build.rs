//! Build script to capture build-time information for version output
//!
//! This generates environment variables at compile time that can be accessed
//! via env!() macro to provide comprehensive version information.

use std::process::Command;

// Note: chrono is available as a build-dependency

fn main() {
    // Git information
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
    {
        let git_hash = String::from_utf8(output.stdout)
            .unwrap_or_default()
            .trim()
            .to_string();
        println!("cargo:rustc-env=LLMSPELL_GIT_HASH={}", git_hash);

        // Get short hash
        if let Ok(output) = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
        {
            let git_short_hash = String::from_utf8(output.stdout)
                .unwrap_or_default()
                .trim()
                .to_string();
            println!("cargo:rustc-env=LLMSPELL_GIT_SHORT_HASH={}", git_short_hash);
        }

        // Check if working tree is dirty
        if let Ok(output) = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
        {
            let is_dirty = !output.stdout.is_empty();
            println!("cargo:rustc-env=LLMSPELL_GIT_DIRTY={}", is_dirty);
        }

        // Get branch name
        if let Ok(output) = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
        {
            let branch = String::from_utf8(output.stdout)
                .unwrap_or_default()
                .trim()
                .to_string();
            println!("cargo:rustc-env=LLMSPELL_GIT_BRANCH={}", branch);
        }

        // Get commit date
        if let Ok(output) = Command::new("git")
            .args(["log", "-1", "--format=%cd", "--date=short"])
            .output()
        {
            let commit_date = String::from_utf8(output.stdout)
                .unwrap_or_default()
                .trim()
                .to_string();
            println!("cargo:rustc-env=LLMSPELL_GIT_COMMIT_DATE={}", commit_date);
        }
    }

    // Build timestamp using date command
    let build_timestamp = if let Ok(output) = Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S%z"])
        .output()
    {
        String::from_utf8(output.stdout)
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        "unknown".to_string()
    };
    println!("cargo:rustc-env=LLMSPELL_BUILD_TIMESTAMP={}", build_timestamp);

    // Build profile (debug/release)
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=LLMSPELL_BUILD_PROFILE={}", profile);

    // Target triple
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=LLMSPELL_TARGET={}", target);

    // Host triple
    let host = std::env::var("HOST").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=LLMSPELL_HOST={}", host);

    // Rust compiler version
    if let Ok(output) = Command::new("rustc")
        .args(["--version"])
        .output()
    {
        let rustc_version = String::from_utf8(output.stdout)
            .unwrap_or_default()
            .trim()
            .to_string();
        println!("cargo:rustc-env=LLMSPELL_RUSTC_VERSION={}", rustc_version);
    }

    // Capture enabled features
    let mut features = Vec::new();
    if cfg!(feature = "common") {
        features.push("common");
    }
    if cfg!(feature = "full") {
        features.push("full");
    }
    if cfg!(feature = "mcp") {
        features.push("mcp");
    }
    if cfg!(feature = "a2a") {
        features.push("a2a");
    }
    if cfg!(feature = "csv-parquet") {
        features.push("csv-parquet");
    }
    if cfg!(feature = "templates") {
        features.push("templates");
    }
    let features_str = if features.is_empty() {
        "default".to_string()
    } else {
        features.join(",")
    };
    println!("cargo:rustc-env=LLMSPELL_FEATURES={}", features_str);

    // Rerun if these change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
}