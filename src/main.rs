use clap::Parser;

fn main() {
    let cli = musketeer::cli::Cli::parse();

    let result = match cli.command {
        musketeer::cli::Command::Init => musketeer::commands::init::run(cli.json),
        musketeer::cli::Command::Run { command } => match command {
            musketeer::cli::RunCommand::New => musketeer::commands::run_new::run(cli.json),
            musketeer::cli::RunCommand::Status { replay } => {
                musketeer::commands::run_status::run(replay, cli.json)
            }
        },
        musketeer::cli::Command::Check { replay } => {
            musketeer::commands::check::run(replay, cli.json)
        }
        musketeer::cli::Command::Packet {
            role,
            replay,
            max_bytes,
        } => musketeer::commands::packet::run(role, replay, max_bytes, cli.json),
        musketeer::cli::Command::Log {
            role,
            kind,
            message,
            replay,
        } => musketeer::commands::log::run(role, kind, message, replay, cli.json),
        musketeer::cli::Command::Verdict {
            role,
            value,
            reason,
            replay,
        } => musketeer::commands::verdict::run(role, value, reason, replay, cli.json),
        musketeer::cli::Command::Migrate { dry_run, force } => {
            musketeer::commands::migrate::run(cli.json, dry_run, force)
        }
    };

    if let Err(err) = result {
        if let Some(me) = err.downcast_ref::<musketeer::error::MusketeerError>() {
            musketeer::output::emit_err(cli.json, None, me.error_code(), &me.to_string());
            std::process::exit(me.exit_code());
        } else {
            musketeer::output::emit_err(cli.json, None, "E_INTERNAL", &err.to_string());
            std::process::exit(50);
        }
    }
}
