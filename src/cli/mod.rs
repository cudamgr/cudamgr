pub mod commands;
pub mod interactive;
pub mod output;

#[cfg(test)]
mod tests;

use crate::error::CudaMgrResult;
use clap::Parser;

#[derive(Parser)]
#[command(name = "cudamgr")]
#[command(about = "A cross-platform CUDA version manager")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: commands::Command,
}

impl Cli {
    pub async fn execute(self) -> CudaMgrResult<()> {
        self.command.execute().await
    }
}
