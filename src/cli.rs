use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "iris")]
#[command(about = "A terminal image viewer with Kitty protocol support")]
#[command(version)]
pub struct Args {
    /// Path to the image file or directory
    pub path: Option<PathBuf>,

    /// Print the image directly without entering interactive mode
    #[arg(long)]
    pub no_interactive: bool,

    /// Enable debug/benchmark logging to stderr
    #[arg(short, long)]
    pub debug: bool,
}
