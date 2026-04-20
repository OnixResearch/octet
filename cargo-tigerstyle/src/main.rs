mod cli;
mod config;
mod engine;
mod output;

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
    let check = cli::parse(std::env::args_os().skip(1))?;
    engine::run(check)
}
