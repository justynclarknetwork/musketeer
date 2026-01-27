use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "musketeer")]
#[command(version)]
#[command(about = "Role-separated execution harness", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init,
    Run {
        #[command(subcommand)]
        command: RunCommand,
    },
    Check {
        #[arg(long)]
        replay: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum RunCommand {
    New,
    Status {
        #[arg(long)]
        replay: Option<String>,
    },
}
