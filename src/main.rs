mod catalog;
mod changes;
mod cli;
mod gradle;
mod graph;
mod inspect;
mod model;
mod plain;
mod releases;
mod tui;

use std::process::ExitCode;

use crate::{
    cli::{Cli, Command},
    inspect::Inspector,
};
use clap::Parser;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Inspect(args)) => match Inspector::new(".", args.catalog, cli.baseline) {
            Ok(inspector) => match inspector.inspect(&args.configuration).await {
                Ok(result) => {
                    print!("{}", plain::render(&result));
                    ExitCode::SUCCESS
                }
                Err(error) => unavailable(&error.to_string()),
            },
            Err(error) => unavailable(&error.to_string()),
        },
        None => match Inspector::new(".", "gradle/libs.versions.toml", cli.baseline) {
            Ok(inspector) => match tui::run(inspector) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => unavailable(&error.to_string()),
            },
            Err(error) => unavailable(&error.to_string()),
        },
    }
}

fn unavailable(message: &str) -> ExitCode {
    eprintln!("gradlens: {message}");
    ExitCode::FAILURE
}
