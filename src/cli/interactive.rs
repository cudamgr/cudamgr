use std::io::{self, Write};

/// Interactive prompts and user confirmations
#[allow(dead_code)]
pub struct Interactive;

impl Interactive {
    /// Ask user for yes/no confirmation
    pub fn confirm(message: &str) -> io::Result<bool> {
        print!("{} (y/N): ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim().to_lowercase();
        Ok(input == "y" || input == "yes")
    }

    /// Ask user to select from a list of options
    pub fn select(message: &str, options: &[String]) -> io::Result<Option<usize>> {
        println!("{}", message);
        for (i, option) in options.iter().enumerate() {
            println!("  {}. {}", i + 1, option);
        }
        print!("Select option (1-{}, or 0 to cancel): ", options.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim().parse::<usize>() {
            Ok(0) => Ok(None),
            Ok(n) if n <= options.len() => Ok(Some(n - 1)),
            _ => {
                println!("Invalid selection");
                Self::select(message, options)
            }
        }
    }

    /// Ask user for text input
    pub fn input(message: &str) -> io::Result<String> {
        print!("{}: ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
}