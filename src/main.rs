#![allow(dead_code)]

mod cli;
mod system;
mod install;
mod version;
mod config;
mod error;

use clap::Parser;
use tracing_subscriber;

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