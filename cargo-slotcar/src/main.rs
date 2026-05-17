mod baseline;
mod cli;
mod config;
mod determinism;
mod engine;
mod fingerprint;
mod migrate;
mod output;
mod roadmap;
mod transcript;

use std::process::ExitCode;

fn main() -> ExitCode {
    match try_main() {
        Ok(code) => ExitCode::from(code as u8),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

fn try_main() -> Result<i32, String> {
    match cli::parse(std::env::args_os())? {
        cli::CommandArgs::Check(check) => engine::run(check),
        cli::CommandArgs::Fingerprint(args) => fingerprint::run(args),
        cli::CommandArgs::Migrate(args) => migrate::run(args),
        cli::CommandArgs::Roadmap(args) => roadmap::run(args),
        cli::CommandArgs::Transcript(args) => transcript::run(args),
        cli::CommandArgs::Help(topic) => {
            println!("{}", cli::help_text(topic));
            Ok(0)
        }
        cli::CommandArgs::Version => {
            println!("cargo-slotcar {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
    }
}
