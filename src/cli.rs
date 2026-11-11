use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "rbuild",
    version,
    about = "Lightning-fast remote Rust compiler using GitHub Codespaces with zero local CPU and persistent SSD caching",
    author = "Praveen Senpai <pvnt20@gmail.com>"
)]
pub struct Cli {
    #[arg(
        help = "Path to the local Rust project or Cargo.toml (defaults to current directory)",
        default_value = "."
    )]
    pub path: PathBuf,

    #[arg(long, help = "Build specific binary target")]
    pub bin: Option<String>,

    #[arg(short, long, help = "Build specific workspace package")]
    pub package: Option<String>,

    #[arg(long, help = "Build in debug mode instead of release")]
    pub debug: bool,

    #[arg(long, help = "Space or comma-separated list of features to activate")]
    pub features: Option<String>,

    #[arg(long, help = "Activate all available features")]
    pub all_features: bool,

    #[arg(long, help = "Do not activate default features")]
    pub no_default_features: bool,

    #[arg(long, help = "Target triple (e.g. x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl)")]
    pub target: Option<String>,

    #[arg(short, long, help = "Install compiled binary directly into ~/.local/bin/")]
    pub install: bool,

    #[arg(short, long, help = "Output directory for compiled binary (default: target/remote/release/)")]
    pub out_dir: Option<PathBuf>,

    #[arg(short, long, help = "Name or display name of the Codespace builder")]
    pub codespace: Option<String>,

    #[arg(long, help = "Run cargo clean on the remote builder before compiling")]
    pub clean: bool,

    #[arg(short, long, help = "Force remote rebuild without skipping unchanged revisions")]
    pub force: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Display status of Codespaces, disk storage, and quota")]
    Status,
    #[command(about = "Stop the builder Codespace to conserve monthly free quota hours")]
    Stop {
        #[arg(short, long, help = "Name of the Codespace to stop")]
        name: Option<String>,
    },
    #[command(about = "Open an interactive SSH shell into the remote builder Codespace")]
    Ssh,
    #[command(about = "Purge remote target directories on the Codespace")]
    Clean,
}
