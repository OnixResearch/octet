mod baseline;
mod cli;
mod config;
mod determinism;
mod engine;
mod evidence_index;
mod fingerprint;
mod migrate;
mod object;
mod output;
mod provider_manifest;
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
        cli::CommandArgs::EvidenceIndex(args) => evidence_index::run(args),
        cli::CommandArgs::Fingerprint(args) => fingerprint::run(args),
        cli::CommandArgs::Migrate(args) => migrate::run(args),
        cli::CommandArgs::Object(args) => object::run(args),
        cli::CommandArgs::ProviderManifest(args) => provider_manifest::run(args),
        cli::CommandArgs::Roadmap(args) => roadmap::run(args),
        cli::CommandArgs::Transcript(args) => transcript::run(args),
        cli::CommandArgs::Help(topic) => {
            println!("{}", cli::help_text(topic));
            Ok(0)
        }
        cli::CommandArgs::Version => {
            println!("cargo-octet {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
    }
}
