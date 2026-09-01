mod cli;
#[allow(dead_code)]
mod model;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::{Cli, Command};

fn main() -> ExitCode {
    match Cli::parse().command {
        Some(Command::Inspect(_)) => unavailable("inspect is not yet wired"),
        None => run_tui(),
    }
}

fn run_tui() -> ExitCode {
    unavailable("interactive mode is not yet available")
}

fn unavailable(message: &str) -> ExitCode {
    eprintln!("gradle-checker: {message}");
    ExitCode::FAILURE
}
