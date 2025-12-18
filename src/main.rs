#![allow(dead_code)]

mod cli;
mod config;
mod error;
mod install;
mod system;
mod version;

use clap::Parser;

use cli::Cli;
use error::CudaMgrResult;

#[tokio::main]
async fn main() -> CudaMgrResult<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    cli.execute().await
}
