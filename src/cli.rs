use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Inspect Gradle version catalogs interactively or from the command line.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Git branch or revision whose catalog is the comparison baseline.
    #[arg(long, global = true)]
    pub baseline: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    Inspect(InspectArgs),
}

#[derive(Args)]
pub struct InspectArgs {
    #[arg(long, default_value = "gradle/libs.versions.toml")]
    pub catalog: PathBuf,

    #[arg(long)]
    pub configuration: String,

    /// Write the plain-text report to this file. Use '-' for stdout.
    #[arg(long, default_value = "-")]
    pub output: PathBuf,

    /// Overwrite an existing output file.
    #[arg(long)]
    pub force: bool,
}
