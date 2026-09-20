use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging across all commands
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compile the current project
    Build,

    /// Run a binary or example of the local package
    Run {
        /// Arguments to pass directly down to the executed binary
        #[arg(last = true)]
        bin_args: Vec<String>,
    },

    /// Remove artifacts that cargo has generated in the past
    Clean,
}

pub fn parse() -> Cli {
    let cli = Cli::parse();
    cli
}
