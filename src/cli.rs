use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "musketeer")]
#[command(version)]
#[command(about = "Role-separated execution harness", long_about = None)]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,

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
    Packet {
        #[arg(long)]
        role: String,
        #[arg(long)]
        replay: Option<String>,
        #[arg(long)]
        max_bytes: Option<usize>,
    },
    Log {
        #[arg(long)]
        role: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        message: String,
        #[arg(long)]
        replay: Option<String>,
    },
    Verdict {
        #[arg(long)]
        role: String,
        #[arg(long)]
        value: String,
        #[arg(long)]
        reason: String,
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
