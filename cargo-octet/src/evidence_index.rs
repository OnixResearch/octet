use std::fs;
use std::path::PathBuf;

use valence_core::evidence_index::EvidenceIndexInput;
use valence_core::evidence_index::check_page_text_with_expected;
use valence_core::evidence_index::render_valence_evidence_index;

use crate::cli::EvidenceIndexArgs;
use crate::cli::EvidenceIndexMode;

const DEFAULT_PAGE: &str = "docs/current-head-valence-evidence.md";

pub fn run(args: EvidenceIndexArgs) -> Result<i32, String> {
    match args.mode {
        EvidenceIndexMode::Render => {
            let input = EvidenceIndexInput {
                capture_time: args
                    .capture_time
                    .ok_or_else(|| "missing --capture-time for evidence index render".to_owned())?,
                payload_commit: args
                    .payload_commit
                    .ok_or_else(|| "missing --payload-commit for evidence index render".to_owned())?,
            };
            print!("{}", render_valence_evidence_index(&input));
            Ok(0)
        }
        EvidenceIndexMode::Check => {
            let page = args.page.unwrap_or_else(|| PathBuf::from(DEFAULT_PAGE));
            let text = fs::read_to_string(&page).map_err(|err| format!("failed to read {}: {err}", page.display()))?;
            let report = check_page_text_with_expected(&text, args.payload_commit, args.capture_time);
            if report.valid {
                println!("valence_evidence_index.valid: PASS");
                println!("valence_evidence_index.issue_count=0");
                Ok(0)
            } else {
                println!("valence_evidence_index.valid: FAIL");
                for code in &report.issue_codes {
                    println!("valence_evidence_index.issue={code}");
                }
                Ok(1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use valence_core::evidence_index::check_page_text;

    use super::*;

    #[test]
    fn evidence_index_cli_uses_valence_core_renderer() {
        let input = EvidenceIndexInput {
            capture_time: "2026-05-23T12:00:00Z".to_owned(),
            payload_commit: "0123456789abcdef0123456789abcdef01234567".to_owned(),
        };
        let rendered = render_valence_evidence_index(&input);
        let report = check_page_text(&rendered);
        assert!(report.valid);
        assert!(report.issue_codes.is_empty());
    }
}
