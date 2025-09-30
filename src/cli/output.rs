use std::io::{self, Write};

/// Utility functions for formatted output and progress indicators
#[allow(dead_code)]
pub struct OutputFormatter;

impl OutputFormatter {
    /// Print a success message with green checkmark
    pub fn success(message: &str) {
        println!("✅ {}", message);
    }

    /// Print an error message with red X
    pub fn error(message: &str) {
        eprintln!("❌ {}", message);
    }

    /// Print a warning message with yellow warning sign
    pub fn warning(message: &str) {
        println!("⚠️  {}", message);
    }

    /// Print an info message with blue info icon
    pub fn info(message: &str) {
        println!("ℹ️  {}", message);
    }

    /// Print a progress message
    pub fn progress(message: &str) {
        print!("⏳ {}...", message);
        io::stdout().flush().unwrap();
    }

    /// Complete a progress message
    pub fn progress_done() {
        println!(" ✅");
    }

    /// Print a section header
    pub fn section(title: &str) {
        println!("\n📋 {}", title);
        println!("{}", "─".repeat(title.len() + 3));
    }
}

/// Progress bar for long-running operations
#[allow(dead_code)]
pub struct ProgressBar {
    total: u64,
    current: u64,
    message: String,
}

impl ProgressBar {
    pub fn new(total: u64, message: String) -> Self {
        Self {
            total,
            current: 0,
            message,
        }
    }

    pub fn update(&mut self, current: u64) {
        self.current = current;
        let percentage = if self.total > 0 {
            (self.current * 100) / self.total
        } else {
            0
        };
        
        print!("\r{}: {}% ({}/{})", self.message, percentage, self.current, self.total);
        io::stdout().flush().unwrap();
    }

    pub fn finish(&self) {
        println!("\r{}: Complete ✅", self.message);
    }
}