use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
    Sarif,
}

impl OutputFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Json => "json",
            Self::Sarif => "sarif",
        }
    }
}

#[derive(Debug)]
pub enum CommandArgs {
    Check(CheckArgs),
    Fingerprint(FingerprintArgs),
    Migrate(MigrateArgs),
    Transcript(TranscriptArgs),
}

#[derive(Debug)]
pub struct CheckArgs {
    pub scope_args: Vec<String>,
    pub cargo_check_args: Option<Vec<String>>,
    pub output_format: OutputFormat,
    pub artifact_dir: Option<PathBuf>,
    pub baseline: Option<PathBuf>,
    pub write_baseline: Option<PathBuf>,
    pub cache: bool,
    pub command_line: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerprintMode {
    Check,
    Write,
    Update,
    Diff,
}

#[derive(Debug)]
pub struct FingerprintArgs {
    pub mode: FingerprintMode,
    pub paths: Vec<PathBuf>,
    pub output_format: OutputFormat,
    pub review_manifest: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrateMode {
    Start,
    Status,
    ApplySafe,
}

#[derive(Debug)]
pub struct MigrateArgs {
    pub mode: MigrateMode,
    pub profile: Option<String>,
    pub output_format: OutputFormat,
    pub manifest: PathBuf,
}

#[derive(Debug)]
pub struct TranscriptArgs {
    pub path: PathBuf,
    pub output_format: OutputFormat,
}

pub fn parse(argv: impl IntoIterator<Item = OsString>) -> Result<CommandArgs, String> {
    let mut argv = argv
        .into_iter()
        .map(|arg| arg.into_string().map_err(|_| "non-utf8 arguments are not supported".to_owned()))
        .collect::<Result<Vec<_>, _>>()?;

    let program = argv.first().cloned().ok_or_else(|| "missing argv[0]".to_owned())?;
    argv.remove(0);

    let invoked_as_cargo_subcommand = argv.first().is_some_and(|arg| matches!(arg.as_str(), "slotcar" | "tigerstyle"));
    let command_line = if invoked_as_cargo_subcommand {
        format_command_line("cargo", &argv)
    } else {
        format_command_line(&program, &argv)
    };

    if invoked_as_cargo_subcommand {
        argv.remove(0);
    }

    match argv.first().map(String::as_str) {
        Some("check") => parse_check(&argv[1..], command_line).map(CommandArgs::Check),
        Some("fingerprint") => parse_fingerprint(&argv[1..]).map(CommandArgs::Fingerprint),
        Some("migrate") => parse_migrate(&argv[1..]).map(CommandArgs::Migrate),
        Some("transcript") => parse_transcript(&argv[1..]).map(CommandArgs::Transcript),
        Some(other) => {
            Err(format!("unknown subcommand `{other}`; expected `check`, `fingerprint`, `migrate`, or `transcript`"))
        }
        None => Err("missing subcommand; expected `check` or `fingerprint`".to_owned()),
    }
}

fn parse_check(args: &[String], command_line: String) -> Result<CheckArgs, String> {
    let mut scope_args = Vec::new();
    let mut cargo_check_args = None;
    let mut output_format = OutputFormat::Human;
    let mut artifact_dir = None;
    let mut baseline = None;
    let mut write_baseline = None;
    let mut cache = false;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            cargo_check_args = Some(args[index + 1..].to_vec());
            break;
        }

        if arg == "--output-format" {
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?;
            output_format = parse_output_format(value)?;
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--output-format=") {
            output_format = parse_output_format(value)?;
            index += 1;
        } else if arg == "--artifact-dir" {
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--artifact-dir`".to_owned())?;
            artifact_dir = Some(PathBuf::from(value));
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--artifact-dir=") {
            artifact_dir = Some(PathBuf::from(value));
            index += 1;
        } else if arg == "--baseline" {
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--baseline`".to_owned())?;
            baseline = Some(PathBuf::from(value));
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--baseline=") {
            baseline = Some(PathBuf::from(value));
            index += 1;
        } else if arg == "--cache" {
            cache = true;
            index += 1;
        } else if arg == "--write-baseline" {
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--write-baseline`".to_owned())?;
            write_baseline = Some(PathBuf::from(value));
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--write-baseline=") {
            write_baseline = Some(PathBuf::from(value));
            index += 1;
        } else {
            match arg.as_str() {
                "-p" | "--package" | "--exclude" => {
                    let value = args.get(index + 1).ok_or_else(|| format!("missing value after `{arg}`"))?;
                    scope_args.push(arg.clone());
                    scope_args.push(value.clone());
                    index += 2;
                }
                "--workspace" => {
                    scope_args.push(arg.clone());
                    index += 1;
                }
                other => return Err(format!("unsupported argument before `--`: `{other}`")),
            }
        }
    }

    Ok(CheckArgs {
        scope_args,
        cargo_check_args,
        output_format,
        artifact_dir,
        baseline,
        write_baseline,
        cache,
        command_line,
    })
}

fn parse_fingerprint(args: &[String]) -> Result<FingerprintArgs, String> {
    let mut mode = None;
    let mut paths = Vec::new();
    let mut output_format = OutputFormat::Human;
    let mut review_manifest = None;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--check" => {
                mode = Some(FingerprintMode::Check);
                index += 1;
            }
            "--write" => {
                mode = Some(FingerprintMode::Write);
                index += 1;
            }
            "--update" => {
                mode = Some(FingerprintMode::Update);
                index += 1;
            }
            "--diff" => {
                mode = Some(FingerprintMode::Diff);
                index += 1;
            }
            "--output-format" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?;
                output_format = parse_output_format(value)?;
                index += 2;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            "--review-manifest" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value after `--review-manifest`".to_owned())?;
                review_manifest = Some(PathBuf::from(value));
                index += 2;
            }
            _ if arg.starts_with("--review-manifest=") => {
                review_manifest = Some(PathBuf::from(arg.trim_start_matches("--review-manifest=")));
                index += 1;
            }
            _ if arg.starts_with('-') => return Err(format!("unsupported fingerprint argument `{arg}`")),
            _ => {
                paths.push(PathBuf::from(arg));
                index += 1;
            }
        }
    }

    Ok(FingerprintArgs {
        mode: mode.unwrap_or(FingerprintMode::Check),
        paths,
        output_format,
        review_manifest,
    })
}

fn parse_migrate(args: &[String]) -> Result<MigrateArgs, String> {
    let mode = match args.first().map(String::as_str) {
        Some("start") => MigrateMode::Start,
        Some("status") => MigrateMode::Status,
        Some("apply-safe") => MigrateMode::ApplySafe,
        Some(other) => {
            return Err(format!("unknown migrate subcommand `{other}`; expected `start`, `status`, or `apply-safe`"));
        }
        None => return Err("missing migrate subcommand; expected `start`, `status`, or `apply-safe`".to_owned()),
    };
    let mut profile = None;
    let mut output_format = OutputFormat::Human;
    let mut manifest = PathBuf::from(".slotcar/migrations/slotcar-migration.json");
    let mut index = 1;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--profile" {
            profile = Some(args.get(index + 1).ok_or_else(|| "missing value after `--profile`".to_owned())?.clone());
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--profile=") {
            profile = Some(value.to_owned());
            index += 1;
        } else if arg == "--format" || arg == "--output-format" {
            output_format =
                parse_output_format(args.get(index + 1).ok_or_else(|| format!("missing value after `{arg}`"))?)?;
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--format=").or_else(|| arg.strip_prefix("--output-format=")) {
            output_format = parse_output_format(value)?;
            index += 1;
        } else if arg == "--manifest" {
            manifest = PathBuf::from(args.get(index + 1).ok_or_else(|| "missing value after `--manifest`".to_owned())?);
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--manifest=") {
            manifest = PathBuf::from(value);
            index += 1;
        } else {
            return Err(format!("unsupported migrate argument `{arg}`"));
        }
    }
    Ok(MigrateArgs {
        mode,
        profile,
        output_format,
        manifest,
    })
}

fn parse_transcript(args: &[String]) -> Result<TranscriptArgs, String> {
    let mut path = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--output-format" || arg == "--format" {
            output_format =
                parse_output_format(args.get(index + 1).ok_or_else(|| format!("missing value after `{arg}`"))?)?;
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--output-format=").or_else(|| arg.strip_prefix("--format=")) {
            output_format = parse_output_format(value)?;
            index += 1;
        } else if arg.starts_with('-') {
            return Err(format!("unsupported transcript argument `{arg}`"));
        } else if path.is_none() {
            path = Some(PathBuf::from(arg));
            index += 1;
        } else {
            return Err(format!("unexpected extra transcript path `{arg}`"));
        }
    }
    Ok(TranscriptArgs {
        path: path.ok_or_else(|| "missing transcript path".to_owned())?,
        output_format,
    })
}

fn parse_output_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "human" => Ok(OutputFormat::Human),
        "json" => Ok(OutputFormat::Json),
        "sarif" => Ok(OutputFormat::Sarif),
        other => Err(format!("unsupported output format `{other}`; expected `human`, `json`, or `sarif`")),
    }
}

fn format_command_line(program: &str, args: &[String]) -> String {
    std::iter::once(program)
        .chain(args.iter().map(String::as_str))
        .map(shell_quote)
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(argument: &str) -> String {
    if !argument.is_empty() && argument.chars().all(is_shell_safe) {
        return argument.to_owned();
    }

    format!("'{}'", argument.replace('\'', "'\"'\"'"))
}

fn is_shell_safe(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '/' | ':' | '=' | '@')
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::CommandArgs;
    use super::FingerprintMode;
    use super::format_command_line;
    use super::parse;

    #[test]
    fn fingerprint_parse_accepts_default_check() {
        let parsed = parse(["cargo", "slotcar", "fingerprint", "src/lib.rs"].map(OsString::from))
            .expect("parse fingerprint args");
        match parsed {
            CommandArgs::Fingerprint(args) => {
                assert_eq!(args.mode, FingerprintMode::Check);
                assert_eq!(args.paths.len(), 1);
            }
            other => panic!("expected fingerprint args, got {other:?}"),
        }
    }

    #[test]
    fn fingerprint_parse_accepts_compatibility_alias() {
        let parsed = parse(["cargo", "tigerstyle", "fingerprint", "src/lib.rs"].map(OsString::from))
            .expect("parse fingerprint args");
        match parsed {
            CommandArgs::Fingerprint(args) => {
                assert_eq!(args.mode, FingerprintMode::Check);
                assert_eq!(args.paths.len(), 1);
            }
            other => panic!("expected fingerprint args, got {other:?}"),
        }
    }

    #[test]
    fn command_line_quoting_matches_contract() {
        assert_eq!(format_command_line("cargo", &["slotcar".to_owned(), "check".to_owned()]), "cargo slotcar check");
        assert_eq!(format_command_line("cargo", &["arg with spaces".to_owned()]), "cargo 'arg with spaces'");
        assert_eq!(format_command_line("cargo", &["can't".to_owned()]), "cargo 'can'\"'\"'t'");
        assert_eq!(format_command_line("cargo", &[String::new()]), "cargo ''");
    }
}
