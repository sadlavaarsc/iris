use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "iris")]
#[command(about = "A terminal image viewer with Kitty protocol support")]
#[command(version)]
pub struct Args {
    /// Path to the image file
    pub path: PathBuf,

    /// Print the image directly without entering interactive mode
    #[arg(long)]
    pub no_interactive: bool,
}
