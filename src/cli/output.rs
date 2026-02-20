use std::io::{self, Write};
use std::time::Instant;

/// Utility functions for formatted output and progress indicators
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

    /// Print a table header
    pub fn table_header(headers: &[&str]) {
        let header_line = headers.join(" | ");
        println!("{}", header_line);
        println!("{}", "─".repeat(header_line.len()));
    }

    /// Print a table row
    pub fn table_row(columns: &[&str]) {
        println!("{}", columns.join(" | "));
    }

    /// Print a status with colored indicator
    pub fn status(label: &str, status: &str, is_ok: bool) {
        let indicator = if is_ok { "✅" } else { "❌" };
        println!("{} {}: {}", indicator, label, status);
    }

    /// Print a spinner for long operations
    pub fn spinner(message: &str) -> Spinner {
        Spinner::new(message.to_string())
    }

    /// Print formatted command usage
    pub fn command_usage(command: &str, description: &str) {
        println!("  {:<20} {}", command, description);
    }

    /// Print a divider line
    pub fn divider() {
        println!("{}", "═".repeat(60));
    }

    /// Print with indentation
    pub fn indent(level: usize, message: &str) {
        let indent = "  ".repeat(level);
        println!("{}{}", indent, message);
    }
}

/// Progress bar for long-running operations
pub struct ProgressBar {
    total: u64,
    current: u64,
    message: String,
    start_time: Instant,
}

impl ProgressBar {
    pub fn new(total: u64, message: String) -> Self {
        Self {
            total,
            current: 0,
            message,
            start_time: Instant::now(),
        }
    }

    /// Update progress bar
    pub fn update(&mut self, current: u64) {
        self.current = current;
        let elapsed = self.start_time.elapsed();

        let percentage = if self.total > 0 {
            (current as f64 / self.total as f64 * 100.0) as u32
        } else {
            0
        };

        let rate = if elapsed.as_secs_f64() > 0.0 {
            self.current as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let bar_width = 40;
        let filled = (percentage as usize * bar_width / 100).min(bar_width);
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);

        print!(
            "\r  [{}] {}% ({}/{}) {:.0} B/s",
            bar, percentage, current, self.total, rate
        );
        io::stdout().flush().unwrap();
    }

    pub fn finish(&self) {
        let elapsed = self.start_time.elapsed();
        println!(
            "\r{}: Complete ✅ (took {:.1}s)",
            self.message,
            elapsed.as_secs_f64()
        );
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }
}

/// Spinner for indeterminate progress
pub struct Spinner {
    message: String,
    frames: Vec<&'static str>,
    current_frame: usize,
    start_time: Instant,
}

impl Spinner {
    pub fn new(message: String) -> Self {
        let spinner = Self {
            message,
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
            start_time: Instant::now(),
        };
        spinner.show();
        spinner
    }

    pub fn tick(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames.len();
        self.show();
    }

    fn show(&self) {
        print!("\r{} {}", self.frames[self.current_frame], self.message);
        io::stdout().flush().unwrap();
    }

    pub fn finish(&self, success: bool) {
        let elapsed = self.start_time.elapsed();
        let icon = if success { "✅" } else { "❌" };
        println!(
            "\r{} {} (took {:.1}s)",
            icon,
            self.message,
            elapsed.as_secs_f64()
        );
    }

    pub fn finish_with_message(&self, message: &str, success: bool) {
        let elapsed = self.start_time.elapsed();
        let icon = if success { "✅" } else { "❌" };
        println!(
            "\r{} {} (took {:.1}s)",
            icon,
            message,
            elapsed.as_secs_f64()
        );
    }
}
