use crate::cli::Cli;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

pub fn run_completion(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "uhegit", &mut io::stdout());
}
