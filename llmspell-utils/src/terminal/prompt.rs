//! Simple terminal prompts for user input
//!
//! Provides lightweight alternatives to dialoguer without external dependencies.

use std::io::{self, Write};

/// Read a line from stdin with a prompt
///
/// # Errors
///
/// Returns an error if reading from stdin fails.
pub fn input(prompt: &str) -> io::Result<String> {
    print!("{prompt}: ");
    io::stdout().flush()?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

/// Read a line from stdin with validation
///
/// # Errors
///
/// Returns an error if reading from stdin fails.
pub fn input_with_validation<F>(prompt: &str, validator: F) -> io::Result<String>
where
    F: Fn(&str) -> Result<(), String>,
{
    loop {
        let response = input(prompt)?;
        match validator(&response) {
            Ok(()) => return Ok(response),
            Err(msg) => {
                eprintln!("Invalid input: {msg}");
            }
        }
    }
}

/// Ask a yes/no question
///
/// # Errors
///
/// Returns an error if reading from stdin fails.
pub fn confirm(prompt: &str, default: bool) -> io::Result<bool> {
    let default_hint = if default { "[Y/n]" } else { "[y/N]" };
    print!("{prompt} {default_hint} ");
    io::stdout().flush()?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let response = buffer.trim().to_lowercase();

    if response.is_empty() {
        return Ok(default);
    }

    Ok(response == "y" || response == "yes")
}

/// Display a menu and get user selection
///
/// # Errors
///
/// Returns an error if reading from stdin fails.
pub fn select(prompt: &str, options: &[&str], default: usize) -> io::Result<usize> {
    println!("{prompt}");
    for (i, option) in options.iter().enumerate() {
        let marker = if i == default { ">" } else { " " };
        println!("{} [{}] {}", marker, i + 1, option);
    }

    loop {
        print!("Select (1-{}) [{}]: ", options.len(), default + 1);
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        let input = buffer.trim();

        if input.is_empty() {
            return Ok(default);
        }

        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= options.len() {
                return Ok(num - 1);
            }
        }

        eprintln!(
            "Invalid selection. Please enter a number between 1 and {}",
            options.len()
        );
    }
}

/// Simple password input (echoes * characters)
///
/// # Errors
///
/// Returns an error if reading from stdin fails.
pub fn password(prompt: &str) -> io::Result<String> {
    // Note: This is a simple implementation that still shows input.
    // For true password hiding, we'd need platform-specific code or a dependency.
    print!("{prompt} (input will be visible): ");
    io::stdout().flush()?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

/// Simple spinner for long operations
pub struct SimpleSpinner {
    frames: Vec<&'static str>,
    current: usize,
    message: String,
    active: bool,
}

impl SimpleSpinner {
    /// Create a new spinner with a message
    #[must_use]
    pub fn new(message: &str) -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current: 0,
            message: message.to_string(),
            active: true,
        }
    }

    /// Create a spinner with custom frames
    #[must_use]
    pub fn with_frames(message: &str, frames: Vec<&'static str>) -> Self {
        Self {
            frames,
            current: 0,
            message: message.to_string(),
            active: true,
        }
    }

    /// Update the message
    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
        if self.active {
            self.tick();
        }
    }

    /// Update the spinner display
    pub fn tick(&mut self) {
        if self.active {
            print!("\r{} {}", self.frames[self.current], self.message);
            io::stdout().flush().ok();
            self.current = (self.current + 1) % self.frames.len();
        }
    }

    /// Temporarily suspend the spinner to print something
    pub fn suspend<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if self.active {
            self.clear();
            let result = f();
            self.tick();
            result
        } else {
            f()
        }
    }

    /// Clear the spinner without deactivating it
    fn clear(&self) {
        if self.active {
            print!("\r{}\r", " ".repeat(self.message.len() + 4));
            io::stdout().flush().ok();
        }
    }

    /// Clear the spinner
    pub fn finish(&mut self) {
        self.clear();
        self.active = false;
    }

    /// Finish with a message
    pub fn finish_with_message(&mut self, msg: &str) {
        self.clear();
        println!("{msg}");
        self.active = false;
    }

    /// Clear and remove the spinner completely
    pub fn finish_and_clear(&mut self) {
        self.finish();
    }
}

/// Async spinner that automatically updates in the background
pub struct AsyncSpinner {
    spinner: std::sync::Arc<std::sync::Mutex<SimpleSpinner>>,
    handle: Option<std::thread::JoinHandle<()>>,
    should_stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl AsyncSpinner {
    /// Create a new async spinner with automatic ticking
    #[must_use]
    pub fn new(message: &str) -> Self {
        let spinner = std::sync::Arc::new(std::sync::Mutex::new(SimpleSpinner::new(message)));
        let spinner_clone = spinner.clone();
        let should_stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let should_stop_clone = should_stop.clone();

        let handle = std::thread::spawn(move || {
            while !should_stop_clone.load(std::sync::atomic::Ordering::Relaxed) {
                if let Ok(mut s) = spinner_clone.lock() {
                    s.tick();
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        Self {
            spinner,
            handle: Some(handle),
            should_stop,
        }
    }

    /// Update the message
    pub fn set_message(&self, message: &str) {
        if let Ok(mut s) = self.spinner.lock() {
            s.set_message(message);
        }
    }

    /// Suspend the spinner to run a closure
    pub fn suspend<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if let Ok(mut s) = self.spinner.lock() {
            s.suspend(f)
        } else {
            f()
        }
    }

    /// Finish with a message
    pub fn finish_with_message(mut self, msg: &str) {
        self.should_stop
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.join().ok();
        }
        if let Ok(mut s) = self.spinner.lock() {
            s.finish_with_message(msg);
        }
    }

    /// Finish and clear
    pub fn finish_and_clear(mut self) {
        self.should_stop
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.join().ok();
        }
        if let Ok(mut s) = self.spinner.lock() {
            s.finish_and_clear();
        }
    }
}

impl Drop for AsyncSpinner {
    fn drop(&mut self) {
        self.should_stop
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.join().ok();
        }
        if let Ok(mut s) = self.spinner.lock() {
            s.finish();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner() {
        let mut spinner = SimpleSpinner::new("Loading");
        spinner.tick();
        assert_eq!(spinner.current, 1);
        spinner.tick();
        assert_eq!(spinner.current, 2);
    }
}
