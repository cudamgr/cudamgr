pub mod commands;
pub mod output;
pub mod interactive;

use clap::Parser;
use crate::error::CudaMgrResult;

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