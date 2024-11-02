mod cli;
mod commands;
mod config;
mod services;
mod types;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::*;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.command
        .unwrap_or(Commands::Watch {
            dir: None,
            output_format: None,
            machine: None,
        })
        .execute()
}
