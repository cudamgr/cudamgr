use crate::cli::output::OutputFormatter;
use std::io::{self, Write};

/// Interactive prompts and user confirmations
pub struct Interactive;

impl Interactive {
    /// Ask user for yes/no confirmation
    pub fn confirm(message: &str) -> io::Result<bool> {
        print!("❓ {} (y/N): ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();
        Ok(input == "y" || input == "yes")
    }

    /// Ask user for yes/no confirmation with default value
    pub fn confirm_with_default(message: &str, default: bool) -> io::Result<bool> {
        let prompt = if default {
            format!("❓ {} (Y/n): ", message)
        } else {
            format!("❓ {} (y/N): ", message)
        };

        print!("{}", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();
        if input.is_empty() {
            Ok(default)
        } else {
            Ok(input == "y" || input == "yes")
        }
    }

    /// Ask user to select from a list of options
    pub fn select(message: &str, options: &[String]) -> io::Result<Option<usize>> {
        loop {
            OutputFormatter::info(message);
            for (i, option) in options.iter().enumerate() {
                println!("  {}. {}", i + 1, option);
            }
            print!("Select option (1-{}, or 0 to cancel): ", options.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().parse::<usize>() {
                Ok(0) => return Ok(None),
                Ok(n) if n <= options.len() => return Ok(Some(n - 1)),
                _ => {
                    OutputFormatter::error("Invalid selection");
                    continue;
                }
            }
        }
    }

    /// Ask user to select from a list with descriptions
    pub fn select_with_description(
        message: &str,
        options: &[(String, String)],
    ) -> io::Result<Option<usize>> {
        loop {
            OutputFormatter::info(message);
            for (i, (option, description)) in options.iter().enumerate() {
                println!("  {}. {} - {}", i + 1, option, description);
            }
            print!("Select option (1-{}, or 0 to cancel): ", options.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().parse::<usize>() {
                Ok(0) => return Ok(None),
                Ok(n) if n <= options.len() => return Ok(Some(n - 1)),
                _ => {
                    OutputFormatter::error("Invalid selection");
                    continue;
                }
            }
        }
    }

    /// Ask user for text input
    pub fn input(message: &str) -> io::Result<String> {
        print!("📝 {}: ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    /// Ask user for text input with validation
    pub fn input_with_validation<F>(message: &str, validator: F) -> io::Result<String>
    where
        F: Fn(&str) -> Result<(), String>,
    {
        loop {
            let input = Self::input(message)?;
            match validator(&input) {
                Ok(()) => return Ok(input),
                Err(error) => {
                    OutputFormatter::error(&error);
                    continue;
                }
            }
        }
    }

    /// Ask user for password input (hidden)
    pub fn password(message: &str) -> io::Result<String> {
        print!("🔒 {}: ", message);
        io::stdout().flush()?;

        // Note: In a real implementation, you'd use a crate like `rpassword`
        // for proper password input hiding. For now, we'll use regular input.
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    /// Show a warning and ask for confirmation
    pub fn warn_and_confirm(warning: &str, confirmation_message: &str) -> io::Result<bool> {
        OutputFormatter::warning(warning);
        Self::confirm(confirmation_message)
    }

    /// Show multiple choice question
    pub fn multiple_choice(question: &str, choices: &[&str]) -> io::Result<Option<usize>> {
        loop {
            OutputFormatter::info(question);
            for (i, choice) in choices.iter().enumerate() {
                println!("  {}. {}", i + 1, choice);
            }
            print!("Select option (1-{}, or 0 to cancel): ", choices.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().parse::<usize>() {
                Ok(0) => return Ok(None),
                Ok(n) if n <= choices.len() => return Ok(Some(n - 1)),
                _ => {
                    OutputFormatter::error("Invalid selection");
                    continue;
                }
            }
        }
    }
}
