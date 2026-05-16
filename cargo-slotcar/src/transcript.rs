use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::cli::OutputFormat;
use crate::cli::TranscriptArgs;
use crate::determinism;

#[derive(Debug, Serialize)]
struct TranscriptReport {
    schema_version: u32,
    path: String,
    transcript_hash: String,
    command_blocks: usize,
    expected_output_blocks: usize,
    status: &'static str,
    failures: Vec<String>,
}

pub fn run(args: TranscriptArgs) -> Result<i32, String> {
    let report = verify(&args.path)?;
    match args.output_format {
        OutputFormat::Human => {
            println!(
                "slotcar transcript {} command_blocks={} expected_output_blocks={} status={}",
                report.path, report.command_blocks, report.expected_output_blocks, report.status,
            );
            for failure in &report.failures {
                println!("failure: {failure}");
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| format!("serialize transcript report: {error}"))?
        ),
        OutputFormat::Sarif => return Err("transcript does not support SARIF output; use human or json".to_owned()),
    }
    Ok(if report.failures.is_empty() { 0 } else { 1 })
}

fn verify(path: &Path) -> Result<TranscriptReport, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut command_blocks = 0;
    let mut expected_output_blocks = 0;
    let mut failures = Vec::new();
    let mut in_fence = false;
    let mut fence_lang = String::new();
    let mut fence_start = 0usize;
    let mut saw_command_in_current_fence = false;

    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;
        if let Some(lang) = line.strip_prefix("```") {
            if in_fence {
                if saw_command_in_current_fence
                    && fence_lang != "slotcar-transcript"
                    && fence_lang != "console"
                    && fence_lang != "sh"
                {
                    failures
                        .push(format!("line {fence_start}: command fence must use slotcar-transcript, console, or sh"));
                }
                in_fence = false;
                fence_lang.clear();
                saw_command_in_current_fence = false;
            } else {
                in_fence = true;
                fence_lang = lang.trim().to_owned();
                fence_start = line_no;
            }
            continue;
        }
        if !in_fence {
            continue;
        }
        if line.trim_start().starts_with("$ ") {
            command_blocks += 1;
            saw_command_in_current_fence = true;
        }
        if line.trim_start().starts_with("=> ") {
            expected_output_blocks += 1;
        }
    }

    if in_fence {
        failures.push(format!("line {fence_start}: unclosed fenced transcript block"));
    }
    if command_blocks == 0 {
        failures.push("transcript contains no `$ ` command lines".to_owned());
    }
    if expected_output_blocks == 0 {
        failures.push("transcript contains no `=> ` expected-output lines".to_owned());
    }

    Ok(TranscriptReport {
        schema_version: 1,
        path: path.display().to_string(),
        transcript_hash: determinism::full_hash(&text),
        command_blocks,
        expected_output_blocks,
        status: if failures.is_empty() { "passed" } else { "failed" },
        failures,
    })
}
