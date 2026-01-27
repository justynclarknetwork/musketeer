use clap::Parser;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = musketeer::cli::Cli::parse();
    match cli.command {
        musketeer::cli::Command::Init => musketeer::commands::init::run(),
        musketeer::cli::Command::Run { command } => match command {
            musketeer::cli::RunCommand::New => musketeer::commands::run_new::run(),
            musketeer::cli::RunCommand::Status { replay } => {
                musketeer::commands::run_status::run(replay)
            }
        },
        musketeer::cli::Command::Check { replay } => musketeer::commands::check::run(replay),
    }
}
