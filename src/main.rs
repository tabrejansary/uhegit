use anyhow::Result;
use clap::Parser;
use uhegit::cli;
use uhegit::commands;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    commands::dispatch(cli)
}
