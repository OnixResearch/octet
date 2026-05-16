mod baseline;
mod cli;
mod config;
mod determinism;
mod engine;
mod fingerprint;
mod migrate;
mod output;
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
        cli::CommandArgs::Transcript(args) => transcript::run(args),
    }
}
