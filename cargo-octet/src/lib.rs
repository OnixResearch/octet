#[cfg(test)]
mod api_contract_rail;
mod artifact;
mod baseline;
mod cli;
mod config;
mod determinism;
mod domain_types;
mod effect_ports;
mod engine;
pub mod evidence;
#[cfg(test)]
mod evidence_contracts;
mod evidence_index;
mod evidence_log;
mod evidence_rails;
#[cfg(test)]
mod evidence_transform;
#[cfg(test)]
mod external_schema_contracts;
mod fingerprint;
mod imported_proof;
#[cfg(test)]
mod legacy_alias_contracts;
mod migrate;
mod migration_sessions;
mod object;
mod object_core;
mod output;
#[cfg(test)]
mod proof_claim_catalog;
mod proof_goal_state;
mod proof_ref;
mod prove;
mod prove_core;
mod provider_manifest;
mod roadmap;
#[cfg(test)]
mod source_boundary_core;
mod transcript;

use std::process::ExitCode;

const CLI_ERROR_EXIT: u8 = 2;

pub fn run_cli_from_env() -> ExitCode {
    match try_main() {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(CLI_ERROR_EXIT)),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(CLI_ERROR_EXIT)
        }
    }
}

fn try_main() -> Result<i32, String> {
    match cli::parse(std::env::args_os())? {
        cli::CommandArgs::Artifact(args) => artifact::run(&args),
        cli::CommandArgs::Check(check) => engine::run(&check),
        cli::CommandArgs::EvidenceIndex(args) => evidence_index::run(args),
        cli::CommandArgs::EvidenceRail(args) => evidence_rails::run(&args),
        cli::CommandArgs::Fingerprint(args) => fingerprint::run(&args),
        cli::CommandArgs::Migrate(args) => migrate::run(args),
        cli::CommandArgs::Object(args) => object::run(&args),
        cli::CommandArgs::ProviderManifest(args) => provider_manifest::run(args),
        cli::CommandArgs::Prove(args) => prove::run(&args),
        cli::CommandArgs::ProofRef(args) => proof_ref::run(&args),
        cli::CommandArgs::Roadmap(args) => roadmap::run(args),
        cli::CommandArgs::Transcript(args) => transcript::run(&args),
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
