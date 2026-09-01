use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Inspect Gradle version catalogs interactively or from the command line.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    Inspect(InspectArgs),
}

#[derive(Args)]
pub struct InspectArgs {
    #[arg(long)]
    pub catalog: PathBuf,

    #[arg(long)]
    pub configuration: String,
}
