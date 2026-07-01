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
    Artifact(ArtifactArgs),
    Check(CheckArgs),
    EvidenceIndex(EvidenceIndexArgs),
    Fingerprint(FingerprintArgs),
    Migrate(MigrateArgs),
    Object(ObjectArgs),
    ProviderManifest(ProviderManifestArgs),
    Prove(ProveArgs),
    Roadmap(crate::roadmap::RoadmapCommand),
    Transcript(TranscriptArgs),
    Help(HelpTopic),
    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    Root,
    Artifact,
    Check,
    EvidenceIndex,
    Fingerprint,
    Migrate,
    Object,
    ProviderManifest,
    Prove,
    Roadmap,
    Transcript,
}

#[derive(Debug)]
pub struct ArtifactArgs {
    pub artifact_dir: PathBuf,
    pub output_format: OutputFormat,
}

#[allow(clippy::struct_field_names)]
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
pub enum EvidenceIndexMode {
    Render,
    Check,
}

#[derive(Debug)]
pub struct EvidenceIndexArgs {
    pub mode: EvidenceIndexMode,
    pub page: Option<PathBuf>,
    pub payload_commit: Option<String>,
    pub capture_time: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectMode {
    Emit,
    Check,
    Update,
    Diff {
        old: PathBuf,
        new: PathBuf,
        pointer_manifest: Option<PathBuf>,
    },
    PointerSet {
        manifest: PathBuf,
        name: String,
        hash: String,
        objects: PathBuf,
        receipt: PathBuf,
    },
    PointerValidate {
        manifest: PathBuf,
        objects: PathBuf,
    },
    BundleExport {
        objects: PathBuf,
        pointers: Option<PathBuf>,
        output: PathBuf,
        replay_command: String,
    },
    CorpusReceipt {
        output: PathBuf,
    },
    CorpusCheck {
        receipt: PathBuf,
    },
    CorpusCaveats,
    CorpusContract,
    SupplementEmit {
        family: SupplementFamilyArg,
        scope: SupplementScopeArg,
    },
    SupplementCheck {
        family: SupplementFamilyArg,
        scope: SupplementScopeArg,
        receipt: PathBuf,
    },
    SupplementFixtures {
        family: SupplementFamilyArg,
        scope: SupplementScopeArg,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupplementFamilyArg {
    Hir,
    Thir,
    Mir,
}

impl SupplementFamilyArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hir => "hir",
            Self::Thir => "thir",
            Self::Mir => "mir",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupplementScopeArg {
    Pilot,
    RustcHirFixtureSpike,
    RustcThirFixtureSpike,
    RustcMirFixtureSpike,
}

impl SupplementScopeArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pilot => "pilot",
            Self::RustcHirFixtureSpike => "rustc-hir-fixture-spike",
            Self::RustcThirFixtureSpike => "rustc-thir-fixture-spike",
            Self::RustcMirFixtureSpike => "rustc-mir-fixture-spike",
        }
    }
}

#[derive(Debug)]
pub struct ObjectArgs {
    pub mode: ObjectMode,
    pub paths: Vec<PathBuf>,
    pub function: Option<String>,
    pub output_format: OutputFormat,
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

#[derive(Debug)]
pub struct ProviderManifestArgs {
    pub mode: ProviderManifestMode,
}

#[derive(Debug)]
pub struct ProveArgs {
    pub registry: Option<PathBuf>,
    pub policy_artifacts_dir: Option<PathBuf>,
    pub source: Vec<PathBuf>,
    pub family: Vec<String>,
    pub artifact_dir: PathBuf,
    pub output_format: OutputFormat,
    pub command_line: String,
    pub list: bool,
    pub explain: bool,
}

#[derive(Debug)]
pub enum ProviderManifestMode {
    Export {
        driver: PathBuf,
        lint_library: PathBuf,
        config: PathBuf,
        standards_policy: Option<PathBuf>,
        rustc: Option<PathBuf>,
        profile: Option<String>,
        required_lints: Vec<String>,
    },
    Validate {
        manifest: PathBuf,
    },
}

pub fn parse(argv: impl IntoIterator<Item = OsString>) -> Result<CommandArgs, String> {
    let mut argv = argv
        .into_iter()
        .map(|arg| arg.into_string().map_err(|_| "non-utf8 arguments are not supported".to_owned()))
        .collect::<Result<Vec<_>, _>>()?;

    let program = argv.first().cloned().ok_or_else(|| "missing argv[0]".to_owned())?;
    argv.remove(0);

    let invoked_as_cargo_subcommand =
        argv.first().is_some_and(|arg| matches!(arg.as_str(), "octet" | "slotcar" | "tigerstyle"));
    let command_line = if invoked_as_cargo_subcommand {
        let mut canonical_argv = argv.clone();
        canonical_argv[0].clear();
        canonical_argv[0].push_str("octet");
        format_command_line("cargo", &canonical_argv)
    } else {
        format_command_line(&program, &argv)
    };

    if invoked_as_cargo_subcommand {
        argv.remove(0);
    }

    match argv.first().map(String::as_str) {
        Some("--help" | "-h" | "help") => Ok(CommandArgs::Help(HelpTopic::Root)),
        Some("--version" | "-V" | "version") => Ok(CommandArgs::Version),
        Some("artifact") if artifact_wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Artifact)),
        Some("artifact") => parse_artifact(&argv[1..]).map(CommandArgs::Artifact),
        Some("check") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Check)),
        Some("check") => parse_check(&argv[1..], command_line).map(CommandArgs::Check),
        Some("evidence-index") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::EvidenceIndex)),
        Some("evidence-index") => parse_evidence_index(&argv[1..]).map(CommandArgs::EvidenceIndex),
        Some("fingerprint") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Fingerprint)),
        Some("fingerprint") => parse_fingerprint(&argv[1..]).map(CommandArgs::Fingerprint),
        Some("migrate") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Migrate)),
        Some("migrate") => parse_migrate(&argv[1..]).map(CommandArgs::Migrate),
        Some("object") if object_wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Object)),
        Some("object") => parse_object(&argv[1..]).map(CommandArgs::Object),
        Some("provider-manifest") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::ProviderManifest)),
        Some("provider-manifest") => parse_provider_manifest(&argv[1..]).map(CommandArgs::ProviderManifest),
        Some("prove") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Prove)),
        Some("prove") => parse_prove(&argv[1..], command_line).map(CommandArgs::Prove),
        Some("roadmap") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Roadmap)),
        Some("roadmap") => crate::roadmap::parse(&argv[1..]).map(CommandArgs::Roadmap),
        Some("transcript") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Transcript)),
        Some("transcript") => parse_transcript(&argv[1..]).map(CommandArgs::Transcript),
        Some(other) => Err(format!(
            "unknown subcommand `{other}`; expected `artifact`, `check`, `evidence-index`, `fingerprint`, `migrate`, `object`, `provider-manifest`, `prove`, `roadmap`, or `transcript`"
        )),
        None => Err(
            "missing subcommand; expected `artifact`, `check`, `evidence-index`, `fingerprint`, `migrate`, `object`, `provider-manifest`, `prove`, `roadmap`, or `transcript`"
                .to_owned(),
        ),
    }
}

fn wants_help(args: &[String]) -> bool {
    matches!(args.first().map(String::as_str), Some("--help" | "-h" | "help"))
}

fn artifact_wants_help(args: &[String]) -> bool {
    wants_help(args) || args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h"))
}

fn object_wants_help(args: &[String]) -> bool {
    wants_help(args) || args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h"))
}

pub fn help_text(topic: HelpTopic) -> &'static str {
    match topic {
        HelpTopic::Root => ROOT_HELP,
        HelpTopic::Artifact => ARTIFACT_HELP,
        HelpTopic::Check => CHECK_HELP,
        HelpTopic::EvidenceIndex => EVIDENCE_INDEX_HELP,
        HelpTopic::Fingerprint => FINGERPRINT_HELP,
        HelpTopic::Migrate => MIGRATE_HELP,
        HelpTopic::Object => OBJECT_HELP,
        HelpTopic::ProviderManifest => PROVIDER_MANIFEST_HELP,
        HelpTopic::Prove => PROVE_HELP,
        HelpTopic::Roadmap => ROADMAP_HELP,
        HelpTopic::Transcript => TRANSCRIPT_HELP,
    }
}

const ROOT_HELP: &str = "Octet operator commands

Usage:
  cargo octet artifact verify --artifact-dir PATH [--output-format human|json]
  cargo octet check [SCOPE] [-- CARGO_CHECK_ARGS]
  cargo octet evidence-index render --payload-commit HASH --capture-time RFC3339
  cargo octet evidence-index check [--page PATH]
  cargo octet fingerprint [--check|--update|--diff] [PATHS...]
  cargo octet migrate <start|status|apply-safe> [--manifest PATH]
  cargo octet object emit|check|update [--function NAME] [PATHS...]
  cargo octet object diff --old OLD.json --new NEW.json [--pointer-manifest PATH]
  cargo octet object pointer set --manifest PATH --name NAME --hash HASH --objects OBJECTS.json --receipt PATH
  cargo octet object pointer validate --manifest PATH --objects OBJECTS.json
  cargo octet object bundle export --objects OBJECTS.json [--pointers POINTERS.json] --output BUNDLE.json --replay-command CMD
  cargo octet object corpus receipt --output RECEIPT.json [PATHS...]
  cargo octet object corpus check --receipt RECEIPT.json [PATHS...]
  cargo octet object corpus caveats [--output-format human|json] [PATHS...]
  cargo octet object corpus contract [--output-format human|json]
  cargo octet object supplement fixtures --family hir --spike rustc-hir-fixture [--output-format human|json]
  cargo octet object supplement fixtures --family thir --spike rustc-thir-fixture [--output-format human|json]
  cargo octet object supplement fixtures --family mir --spike rustc-mir-fixture [--output-format human|json]
  cargo octet object supplement emit --family hir|thir|mir [--spike rustc-hir-fixture|rustc-thir-fixture|rustc-mir-fixture] [--fixture-shape checked-arithmetic|branch-control-flow|call-return] [--function NAME] [PATHS...]
  cargo octet object supplement check --family hir|thir|mir [--spike rustc-hir-fixture|rustc-thir-fixture|rustc-mir-fixture] [--fixture-shape checked-arithmetic|branch-control-flow|call-return] --receipt RECEIPT.json [--function NAME] [PATHS...]
  cargo octet provider-manifest export --driver PATH --lint-library PATH --config PATH [--standards-policy PATH]
  cargo octet provider-manifest validate --manifest PATH
  cargo octet prove --policy-artifacts-dir PATH --source PATH --artifact-dir PATH [--list] [--explain] [--output-format human|json]
  cargo octet roadmap <artifact|capsule|ability|pointer|session|pure-cache> ...
  cargo octet transcript <PATH>
  cargo octet --version

Release smoke surfaces: fingerprint identity review, migrate manifests, check --cache lint-cache status, effect-summary output in summaries/artifacts, and executable transcript validation.

Nix: nix run path:/path/to/octet#cargo-octet -- check
";

const ARTIFACT_HELP: &str = "cargo octet artifact verify

Options: --artifact-dir PATH, --output-format human|json.

Verifies a saved cargo octet check artifact directory without mutating it. The verifier recomputes BLAKE3 hashes, validates provenance coverage, rejects stale sidecars, and reports deterministic human or JSON output.
";

const CHECK_HELP: &str = "cargo octet check

Options: --output-format human|json|sarif, --artifact-dir PATH, --baseline PATH, --write-baseline PATH, --cache.

Artifacts include summary.txt effect-summary output, findings JSON/SARIF, and cache_status fields when --cache enables the lint-cache.
";

const EVIDENCE_INDEX_HELP: &str = "cargo octet evidence-index

Subcommands:
  render --payload-commit HASH --capture-time RFC3339
  check [--page docs/current-head-valence-evidence.md]

Renders or checks the generated current-head Valence evidence Markdown index. Check mode fails closed on stale payload, missing rail/pass signal, or weakened non-claim boundaries.
";

const FINGERPRINT_HELP: &str = "cargo octet fingerprint

Modes: --check, --write, --update, --diff. Use --review-manifest PATH to save review records. Fingerprint receipts use stable semantic identity and BLAKE3 hashes.
";

const MIGRATE_HELP: &str = "cargo octet migrate

Subcommands: start, status, apply-safe. Migration manifests are deterministic and safe-reviewable before applying.
";

const OBJECT_HELP: &str = "cargo octet object

Subcommands:
  emit [--function NAME] [PATHS...] --output-format human|json
  check [--function NAME] [PATHS...] --output-format human|json
  update [--function NAME] [PATHS...] --output-format human|json
  diff --old OLD.json --new NEW.json [--pointer-manifest PATH] --output-format human|json
  pointer set --manifest PATH --name NAME --hash HASH --objects OBJECTS.json --receipt PATH
  pointer validate --manifest PATH --objects OBJECTS.json
  bundle export --objects OBJECTS.json [--pointers POINTERS.json] --output BUNDLE.json --replay-command CMD
  corpus receipt --output RECEIPT.json [PATHS...]
  corpus check --receipt RECEIPT.json [PATHS...] --output-format human|json
  corpus caveats [PATHS...] --output-format human|json
  corpus contract --output-format human|json
  supplement fixtures --family hir --spike rustc-hir-fixture --output-format human|json
  supplement fixtures --family thir --spike rustc-thir-fixture --output-format human|json
  supplement fixtures --family mir --spike rustc-mir-fixture --output-format human|json
  supplement emit --family hir|thir|mir [--spike rustc-hir-fixture|rustc-thir-fixture|rustc-mir-fixture] [--fixture-shape checked-arithmetic|branch-control-flow|call-return] [--function NAME] [PATHS...] --output-format human|json
  supplement check --family hir|thir|mir [--spike rustc-hir-fixture|rustc-thir-fixture|rustc-mir-fixture] [--fixture-shape checked-arithmetic|branch-control-flow|call-return] --receipt RECEIPT.json [--function NAME] [PATHS...] --output-format human|json

Function objects are source-token review/provenance evidence. HIR/MIR supplements attach to v1 identity; THIR default supplements are fail-closed unsupported-planned evidence, with only selected rustc-thir-fixture and rustc-mir-fixture cases reporting pilot-supported. Supplements are not correctness proofs.
";

const PROVIDER_MANIFEST_HELP: &str = "cargo octet provider-manifest

Subcommands:
  export --driver PATH --lint-library PATH --config PATH [--standards-policy PATH] [--rustc PATH] [--profile NAME] [--required-lint NAME]
  validate --manifest PATH

Exports or validates the deterministic Octet compiler-policy provider manifest consumed by downstream build systems such as Mantle. The manifest identifies the Dylint driver, Octet lint library, lint config, optional standards policy artifact, compatible rustc identity, required environment, waiver contract, and bounded non-claims.
";

const PROVE_HELP: &str = "cargo octet prove

Options: --registry PATH or --policy-artifacts-dir PATH, --source PATH (repeatable), --family NAME, --artifact-dir PATH, --list, --explain, --output-format human|json.

Runs selected standards proof rails through argv-based commands, writes structured proof receipts under the artifact directory, emits formal_proof provenance only for validated receipts with verifier identity and proof artifact identity, and keeps ordinary cargo octet check behavior unchanged. Use --list to show selected rails without executing them and --explain to prefix failure diagnostics with remediation context.
";

const ROADMAP_HELP: &str = "cargo octet roadmap

Unison-inspired agent roadmap fixtures: content-addressed artifacts, ability manifests, name-to-hash pointers, typed session records, structured todos, and pure-result cache receipts.

Actions:
  artifact store|inspect|replay
  capsule capture|inspect|replay
  ability registry|summary|enforce
  pointer set|inspect|rollback
  session check|migrate|todo
  pure-cache check
";

const TRANSCRIPT_HELP: &str = "cargo octet transcript <PATH>

Validates executable transcript command blocks and expected-output assertions for docs-example and release smoke artifacts.
";

fn parse_artifact(args: &[String]) -> Result<ArtifactArgs, String> {
    match args.first().map(String::as_str) {
        Some("verify") => parse_artifact_verify(&args[1..]),
        Some(other) => Err(format!("unknown artifact subcommand `{other}`; expected `verify`")),
        None => Err("missing artifact subcommand; expected `verify`".to_owned()),
    }
}

fn parse_artifact_verify(args: &[String]) -> Result<ArtifactArgs, String> {
    const FLAG_AND_VALUE_ARGS: usize = 2;

    let mut artifact_dir = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--artifact-dir" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value after `--artifact-dir`".to_owned())?;
                artifact_dir = Some(PathBuf::from(value));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--output-format" | "--format" => {
                let value = args.get(index + 1).ok_or_else(|| format!("missing value after `{arg}`"))?;
                output_format = parse_output_format(value)?;
                index += FLAG_AND_VALUE_ARGS;
            }
            _ if arg.starts_with("--artifact-dir=") => {
                artifact_dir = Some(PathBuf::from(arg.trim_start_matches("--artifact-dir=")));
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with("--format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--format="))?;
                index += 1;
            }
            _ if arg.starts_with('-') => return Err(format!("unsupported artifact verify argument `{arg}`")),
            _ => return Err(format!("unexpected artifact verify path `{arg}`; use `--artifact-dir`")),
        }
    }

    Ok(ArtifactArgs {
        artifact_dir: artifact_dir.ok_or_else(|| "artifact verify requires `--artifact-dir PATH`".to_owned())?,
        output_format,
    })
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

fn parse_evidence_index(args: &[String]) -> Result<EvidenceIndexArgs, String> {
    let mode = match args.first().map(String::as_str) {
        Some("render") => EvidenceIndexMode::Render,
        Some("check") => EvidenceIndexMode::Check,
        Some(other) => {
            return Err(format!("unknown evidence-index subcommand `{other}`; expected `render` or `check`"));
        }
        None => return Err("missing evidence-index subcommand; expected `render` or `check`".to_owned()),
    };

    let mut page = None;
    let mut payload_commit = None;
    let mut capture_time = None;
    let mut index = 1;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--page" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value after `--page`".to_owned())?;
                page = Some(PathBuf::from(value));
                index += 2;
            }
            "--payload-commit" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value after `--payload-commit`".to_owned())?;
                payload_commit = Some(value.clone());
                index += 2;
            }
            "--capture-time" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value after `--capture-time`".to_owned())?;
                capture_time = Some(value.clone());
                index += 2;
            }
            _ if arg.starts_with("--page=") => {
                page = Some(PathBuf::from(arg.trim_start_matches("--page=")));
                index += 1;
            }
            _ if arg.starts_with("--payload-commit=") => {
                payload_commit = Some(arg.trim_start_matches("--payload-commit=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--capture-time=") => {
                capture_time = Some(arg.trim_start_matches("--capture-time=").to_owned());
                index += 1;
            }
            _ => return Err(format!("unsupported evidence-index argument `{arg}`")),
        }
    }

    Ok(EvidenceIndexArgs {
        mode,
        page,
        payload_commit,
        capture_time,
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
    let mut manifest = PathBuf::from(".octet/migrations/octet-migration.json");
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

fn parse_object(args: &[String]) -> Result<ObjectArgs, String> {
    match args.first().map(String::as_str) {
        Some("emit") => parse_object_scan(ObjectMode::Emit, &args[1..]),
        Some("check") => parse_object_scan(ObjectMode::Check, &args[1..]),
        Some("update") => parse_object_scan(ObjectMode::Update, &args[1..]),
        Some("diff") => parse_object_diff(&args[1..]),
        Some("pointer") => parse_object_pointer(&args[1..]),
        Some("bundle") => parse_object_bundle(&args[1..]),
        Some("corpus") => parse_object_corpus(&args[1..]),
        Some("supplement") => parse_object_supplement(&args[1..]),
        Some(other) => Err(format!(
            "unknown object subcommand `{other}`; expected `emit`, `check`, `update`, `diff`, `pointer`, `bundle`, `corpus`, or `supplement`"
        )),
        None => Err(
            "missing object subcommand; expected `emit`, `check`, `update`, `diff`, `pointer`, `bundle`, `corpus`, or `supplement`"
                .to_owned(),
        ),
    }
}

fn parse_object_scan(mode: ObjectMode, args: &[String]) -> Result<ObjectArgs, String> {
    let mut paths = Vec::new();
    let mut function = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--function" => {
                function =
                    Some(args.get(index + 1).ok_or_else(|| "missing value after `--function`".to_owned())?.clone());
                index += 2;
            }
            _ if arg.starts_with("--function=") => {
                function = Some(arg.trim_start_matches("--function=").to_owned());
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
            _ if arg.starts_with('-') => return Err(format!("unsupported object argument `{arg}`")),
            _ => {
                paths.push(PathBuf::from(arg));
                index += 1;
            }
        }
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }
    Ok(ObjectArgs {
        mode,
        paths,
        function,
        output_format,
    })
}

fn parse_object_diff(args: &[String]) -> Result<ObjectArgs, String> {
    let mut old = None;
    let mut new = None;
    let mut pointer_manifest = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--old" => {
                old = Some(PathBuf::from(args.get(index + 1).ok_or_else(|| "missing value after `--old`".to_owned())?));
                index += 2;
            }
            "--new" => {
                new = Some(PathBuf::from(args.get(index + 1).ok_or_else(|| "missing value after `--new`".to_owned())?));
                index += 2;
            }
            "--pointer-manifest" => {
                pointer_manifest = Some(PathBuf::from(
                    args.get(index + 1).ok_or_else(|| "missing value after `--pointer-manifest`".to_owned())?,
                ));
                index += 2;
            }
            "--output-format" => {
                output_format = parse_output_format(
                    args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                )?;
                index += 2;
            }
            _ if arg.starts_with("--old=") => {
                old = Some(PathBuf::from(arg.trim_start_matches("--old=")));
                index += 1;
            }
            _ if arg.starts_with("--new=") => {
                new = Some(PathBuf::from(arg.trim_start_matches("--new=")));
                index += 1;
            }
            _ if arg.starts_with("--pointer-manifest=") => {
                pointer_manifest = Some(PathBuf::from(arg.trim_start_matches("--pointer-manifest=")));
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ => return Err(format!("unsupported object diff argument `{arg}`")),
        }
    }
    Ok(ObjectArgs {
        mode: ObjectMode::Diff {
            old: old.ok_or_else(|| "missing `--old`".to_owned())?,
            new: new.ok_or_else(|| "missing `--new`".to_owned())?,
            pointer_manifest,
        },
        paths: Vec::new(),
        function: None,
        output_format,
    })
}

fn parse_object_pointer(args: &[String]) -> Result<ObjectArgs, String> {
    match args.first().map(String::as_str) {
        Some("set") => {
            let manifest = PathBuf::from(required_arg(&args[1..], "--manifest")?);
            let name = required_arg(&args[1..], "--name")?;
            let hash = required_arg(&args[1..], "--hash")?;
            let objects = PathBuf::from(required_arg(&args[1..], "--objects")?);
            let receipt = PathBuf::from(required_arg(&args[1..], "--receipt")?);
            Ok(ObjectArgs {
                mode: ObjectMode::PointerSet {
                    manifest,
                    name,
                    hash,
                    objects,
                    receipt,
                },
                paths: Vec::new(),
                function: None,
                output_format: OutputFormat::Json,
            })
        }
        Some("validate") => Ok(ObjectArgs {
            mode: ObjectMode::PointerValidate {
                manifest: PathBuf::from(required_arg(&args[1..], "--manifest")?),
                objects: PathBuf::from(required_arg(&args[1..], "--objects")?),
            },
            paths: Vec::new(),
            function: None,
            output_format: OutputFormat::Json,
        }),
        Some(other) => Err(format!("unknown object pointer action `{other}`; expected `set` or `validate`")),
        None => Err("missing object pointer action; expected `set` or `validate`".to_owned()),
    }
}

fn parse_object_bundle(args: &[String]) -> Result<ObjectArgs, String> {
    match args.first().map(String::as_str) {
        Some("export") => Ok(ObjectArgs {
            mode: ObjectMode::BundleExport {
                objects: PathBuf::from(required_arg(&args[1..], "--objects")?),
                pointers: optional_arg(&args[1..], "--pointers").map(PathBuf::from),
                output: PathBuf::from(required_arg(&args[1..], "--output")?),
                replay_command: required_arg(&args[1..], "--replay-command")?,
            },
            paths: Vec::new(),
            function: None,
            output_format: OutputFormat::Json,
        }),
        Some(other) => Err(format!("unknown object bundle action `{other}`; expected `export`")),
        None => Err("missing object bundle action; expected `export`".to_owned()),
    }
}

#[allow(clippy::too_many_lines)]
fn parse_object_corpus(args: &[String]) -> Result<ObjectArgs, String> {
    match args.first().map(String::as_str) {
        Some("receipt") => {
            let mut paths = Vec::new();
            let mut output = None;
            let mut index = 1;
            while index < args.len() {
                let arg = &args[index];
                if arg == "--output" {
                    output = Some(PathBuf::from(
                        args.get(index + 1).ok_or_else(|| "missing value after `--output`".to_owned())?,
                    ));
                    index += 2;
                } else if let Some(value) = arg.strip_prefix("--output=") {
                    output = Some(PathBuf::from(value));
                    index += 1;
                } else if arg.starts_with('-') {
                    return Err(format!("unsupported object corpus receipt argument `{arg}`"));
                } else {
                    paths.push(PathBuf::from(arg));
                    index += 1;
                }
            }
            if paths.is_empty() {
                paths.push(PathBuf::from("."));
            }
            Ok(ObjectArgs {
                mode: ObjectMode::CorpusReceipt {
                    output: output.ok_or_else(|| "missing `--output`".to_owned())?,
                },
                paths,
                function: None,
                output_format: OutputFormat::Json,
            })
        }
        Some("check") => {
            let mut paths = Vec::new();
            let mut receipt = None;
            let mut output_format = OutputFormat::Human;
            let mut index = 1;
            while index < args.len() {
                let arg = &args[index];
                if arg == "--receipt" {
                    receipt = Some(PathBuf::from(
                        args.get(index + 1).ok_or_else(|| "missing value after `--receipt`".to_owned())?,
                    ));
                    index += 2;
                } else if let Some(value) = arg.strip_prefix("--receipt=") {
                    receipt = Some(PathBuf::from(value));
                    index += 1;
                } else if arg == "--output-format" {
                    output_format = parse_output_format(
                        args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                    )?;
                    index += 2;
                } else if let Some(value) = arg.strip_prefix("--output-format=") {
                    output_format = parse_output_format(value)?;
                    index += 1;
                } else if arg.starts_with('-') {
                    return Err(format!("unsupported object corpus check argument `{arg}`"));
                } else {
                    paths.push(PathBuf::from(arg));
                    index += 1;
                }
            }
            if paths.is_empty() {
                paths.push(PathBuf::from("."));
            }
            Ok(ObjectArgs {
                mode: ObjectMode::CorpusCheck {
                    receipt: receipt.ok_or_else(|| "missing `--receipt`".to_owned())?,
                },
                paths,
                function: None,
                output_format,
            })
        }
        Some("caveats") => {
            let mut paths = Vec::new();
            let mut output_format = OutputFormat::Human;
            let mut index = 1;
            while index < args.len() {
                let arg = &args[index];
                if arg == "--output-format" {
                    output_format = parse_output_format(
                        args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                    )?;
                    index += 2;
                } else if let Some(value) = arg.strip_prefix("--output-format=") {
                    output_format = parse_output_format(value)?;
                    index += 1;
                } else if arg.starts_with('-') {
                    return Err(format!("unsupported object corpus caveats argument `{arg}`"));
                } else {
                    paths.push(PathBuf::from(arg));
                    index += 1;
                }
            }
            if paths.is_empty() {
                paths.push(PathBuf::from("."));
            }
            Ok(ObjectArgs {
                mode: ObjectMode::CorpusCaveats,
                paths,
                function: None,
                output_format,
            })
        }
        Some("contract") => {
            let mut output_format = OutputFormat::Human;
            let mut index = 1;
            while index < args.len() {
                let arg = &args[index];
                if arg == "--output-format" {
                    output_format = parse_output_format(
                        args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                    )?;
                    index += 2;
                } else if let Some(value) = arg.strip_prefix("--output-format=") {
                    output_format = parse_output_format(value)?;
                    index += 1;
                } else {
                    return Err(format!("unsupported object corpus contract argument `{arg}`"));
                }
            }
            Ok(ObjectArgs {
                mode: ObjectMode::CorpusContract,
                paths: Vec::new(),
                function: None,
                output_format,
            })
        }
        Some(other) => Err(format!(
            "unknown object corpus action `{other}`; expected `receipt`, `check`, `caveats`, or `contract`"
        )),
        None => Err("missing object corpus action; expected `receipt`, `check`, `caveats`, or `contract`".to_owned()),
    }
}

fn parse_object_supplement(args: &[String]) -> Result<ObjectArgs, String> {
    match args.first().map(String::as_str) {
        Some("fixtures") => parse_object_supplement_fixtures(&args[1..]),
        Some("emit") => parse_object_supplement_scan(true, &args[1..]),
        Some("check") => parse_object_supplement_scan(false, &args[1..]),
        Some(other) => {
            Err(format!("unknown object supplement action `{other}`; expected `fixtures`, `emit`, or `check`"))
        }
        None => Err("missing object supplement action; expected `fixtures`, `emit`, or `check`".to_owned()),
    }
}

fn parse_object_supplement_fixtures(args: &[String]) -> Result<ObjectArgs, String> {
    let mut family = None;
    let mut scope = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--family" => {
                family = Some(parse_supplement_family(
                    args.get(index + 1).ok_or_else(|| "missing value after `--family`".to_owned())?,
                )?);
                index += 2;
            }
            "--spike" => {
                scope = Some(parse_supplement_spike_scope(
                    args.get(index + 1).ok_or_else(|| "missing value after `--spike`".to_owned())?,
                )?);
                index += 2;
            }
            "--output-format" => {
                output_format = parse_output_format(
                    args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                )?;
                index += 2;
            }
            _ if arg.starts_with("--family=") => {
                family = Some(parse_supplement_family(arg.trim_start_matches("--family="))?);
                index += 1;
            }
            _ if arg.starts_with("--spike=") => {
                scope = Some(parse_supplement_spike_scope(arg.trim_start_matches("--spike="))?);
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unsupported object supplement fixtures argument `{arg}`"));
            }
            _ => return Err(format!("unexpected object supplement fixtures path `{arg}`")),
        }
    }
    let family = family.ok_or_else(|| {
        "object supplement fixtures requires `--family hir`, `--family thir`, or `--family mir`".to_owned()
    })?;
    let scope = scope.ok_or_else(|| {
        "object supplement fixtures requires `--spike rustc-hir-fixture`, `--spike rustc-thir-fixture`, or `--spike rustc-mir-fixture`".to_owned()
    })?;
    match (family, scope) {
        (SupplementFamilyArg::Hir, SupplementScopeArg::RustcHirFixtureSpike)
        | (SupplementFamilyArg::Thir, SupplementScopeArg::RustcThirFixtureSpike)
        | (SupplementFamilyArg::Mir, SupplementScopeArg::RustcMirFixtureSpike) => {}
        (SupplementFamilyArg::Hir, _) => {
            return Err("object supplement fixtures for `--family hir` requires `--spike rustc-hir-fixture`".to_owned());
        }
        (SupplementFamilyArg::Thir, _) => {
            return Err(
                "object supplement fixtures for `--family thir` requires `--spike rustc-thir-fixture`".to_owned()
            );
        }
        (SupplementFamilyArg::Mir, _) => {
            return Err("object supplement fixtures for `--family mir` requires `--spike rustc-mir-fixture`".to_owned());
        }
    }
    Ok(ObjectArgs {
        mode: ObjectMode::SupplementFixtures { family, scope },
        paths: Vec::new(),
        function: None,
        output_format,
    })
}

#[allow(clippy::too_many_lines)]
fn parse_object_supplement_scan(emit: bool, args: &[String]) -> Result<ObjectArgs, String> {
    let mut paths = Vec::new();
    let mut function = None;
    let mut family = None;
    let mut scope = SupplementScopeArg::Pilot;
    let mut receipt = None;
    let mut output_format = OutputFormat::Human;
    let mut fixture_shape = None;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--family" => {
                family = Some(parse_supplement_family(
                    args.get(index + 1).ok_or_else(|| "missing value after `--family`".to_owned())?,
                )?);
                index += 2;
            }
            "--function" => {
                function =
                    Some(args.get(index + 1).ok_or_else(|| "missing value after `--function`".to_owned())?.clone());
                index += 2;
            }
            "--receipt" => {
                receipt = Some(PathBuf::from(
                    args.get(index + 1).ok_or_else(|| "missing value after `--receipt`".to_owned())?,
                ));
                index += 2;
            }
            "--spike" => {
                scope = parse_supplement_spike_scope(
                    args.get(index + 1).ok_or_else(|| "missing value after `--spike`".to_owned())?,
                )?;
                index += 2;
            }
            "--fixture-shape" => {
                fixture_shape = Some(
                    args.get(index + 1).ok_or_else(|| "missing value after `--fixture-shape`".to_owned())?.clone(),
                );
                index += 2;
            }
            "--output-format" => {
                output_format = parse_output_format(
                    args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                )?;
                index += 2;
            }
            _ if arg.starts_with("--family=") => {
                family = Some(parse_supplement_family(arg.trim_start_matches("--family="))?);
                index += 1;
            }
            _ if arg.starts_with("--function=") => {
                function = Some(arg.trim_start_matches("--function=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--receipt=") => {
                receipt = Some(PathBuf::from(arg.trim_start_matches("--receipt=")));
                index += 1;
            }
            _ if arg.starts_with("--spike=") => {
                scope = parse_supplement_spike_scope(arg.trim_start_matches("--spike="))?;
                index += 1;
            }
            _ if arg.starts_with("--fixture-shape=") => {
                fixture_shape = Some(arg.trim_start_matches("--fixture-shape=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with('-') => return Err(format!("unsupported object supplement argument `{arg}`")),
            _ => {
                paths.push(PathBuf::from(arg));
                index += 1;
            }
        }
    }
    let fixture_shape_was_provided = fixture_shape.is_some();
    if let Some(shape) = fixture_shape {
        if !matches!(
            scope,
            SupplementScopeArg::RustcHirFixtureSpike
                | SupplementScopeArg::RustcThirFixtureSpike
                | SupplementScopeArg::RustcMirFixtureSpike
        ) {
            return Err(
                "`--fixture-shape` requires `--spike rustc-hir-fixture`, `--spike rustc-thir-fixture`, or `--spike rustc-mir-fixture`".to_owned()
            );
        }
        if !paths.is_empty() || function.is_some() {
            return Err("`--fixture-shape` cannot be combined with explicit fixture paths or `--function`".to_owned());
        }
        if matches!(scope, SupplementScopeArg::RustcMirFixtureSpike)
            && !matches!(shape.trim(), "checked-arithmetic" | "call-return" | "branch-control-flow")
        {
            return Err(
                "`--spike rustc-mir-fixture` currently supports only `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`".to_owned()
            );
        }
        let selected = parse_supplement_fixture_shape(scope, &shape)?;
        paths.push(selected.path);
        function = Some(selected.function);
    }
    let family = family.ok_or_else(|| "missing `--family`".to_owned())?;
    match (family, scope) {
        (SupplementFamilyArg::Hir | SupplementFamilyArg::Mir, SupplementScopeArg::RustcThirFixtureSpike) => {
            return Err("rustc THIR fixture spike only supports `--family thir`".to_owned());
        }
        (SupplementFamilyArg::Thir | SupplementFamilyArg::Mir, SupplementScopeArg::RustcHirFixtureSpike) => {
            return Err("rustc HIR fixture spike only supports `--family hir`".to_owned());
        }
        (SupplementFamilyArg::Hir | SupplementFamilyArg::Thir, SupplementScopeArg::RustcMirFixtureSpike) => {
            return Err("rustc MIR fixture spike only supports `--family mir`".to_owned());
        }
        _ => {}
    }
    if matches!((family, scope), (SupplementFamilyArg::Thir, SupplementScopeArg::RustcThirFixtureSpike))
        && !fixture_shape_was_provided
    {
        return Err(
            "`--family thir --spike rustc-thir-fixture` requires explicit `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`"
                .to_owned(),
        );
    }
    if matches!((family, scope), (SupplementFamilyArg::Mir, SupplementScopeArg::RustcMirFixtureSpike))
        && !fixture_shape_was_provided
    {
        return Err("`--family mir --spike rustc-mir-fixture` requires explicit `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`"
            .to_owned());
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }
    let mode = if emit {
        ObjectMode::SupplementEmit { family, scope }
    } else {
        ObjectMode::SupplementCheck {
            family,
            scope,
            receipt: receipt.ok_or_else(|| "missing `--receipt`".to_owned())?,
        }
    };
    Ok(ObjectArgs {
        mode,
        paths,
        function,
        output_format,
    })
}

fn parse_supplement_family(value: &str) -> Result<SupplementFamilyArg, String> {
    match value {
        "hir" => Ok(SupplementFamilyArg::Hir),
        "thir" => Ok(SupplementFamilyArg::Thir),
        "mir" => Ok(SupplementFamilyArg::Mir),
        other => Err(format!("unsupported supplement family `{other}`; expected `hir`, `thir`, or `mir`")),
    }
}

fn parse_supplement_spike_scope(value: &str) -> Result<SupplementScopeArg, String> {
    match value {
        "rustc-hir-fixture" | "rustc-hir-fixture-spike" => Ok(SupplementScopeArg::RustcHirFixtureSpike),
        "rustc-thir-fixture" | "rustc-thir-fixture-spike" => Ok(SupplementScopeArg::RustcThirFixtureSpike),
        "rustc-mir-fixture" | "rustc-mir-fixture-spike" => Ok(SupplementScopeArg::RustcMirFixtureSpike),
        other => Err(format!(
            "unsupported supplement spike `{other}`; expected `rustc-hir-fixture`, `rustc-thir-fixture`, or `rustc-mir-fixture`"
        )),
    }
}

struct SupplementFixtureSelection {
    path: PathBuf,
    function: String,
}

fn parse_supplement_fixture_shape(
    scope: SupplementScopeArg,
    value: &str,
) -> Result<SupplementFixtureSelection, String> {
    match (scope, value) {
        (SupplementScopeArg::RustcHirFixtureSpike, "checked" | "checked-arithmetic") => {
            Ok(SupplementFixtureSelection {
                path: PathBuf::from("cargo-octet/tests/fixtures/hir-spike/checked.rs"),
                function: "checked".to_owned(),
            })
        }
        (SupplementScopeArg::RustcHirFixtureSpike, "branch" | "branch-control-flow" | "clamp") => {
            Ok(SupplementFixtureSelection {
                path: PathBuf::from("cargo-octet/tests/fixtures/hir-spike/clamp.rs"),
                function: "clamp_to_limit".to_owned(),
            })
        }
        (SupplementScopeArg::RustcThirFixtureSpike, "checked" | "checked-arithmetic") => {
            Ok(SupplementFixtureSelection {
                path: PathBuf::from("cargo-octet/tests/fixtures/thir-spike/checked.rs"),
                function: "checked".to_owned(),
            })
        }
        (SupplementScopeArg::RustcThirFixtureSpike, "call-return") => Ok(SupplementFixtureSelection {
            path: PathBuf::from("cargo-octet/tests/fixtures/thir-spike/call_return.rs"),
            function: "call_then_return".to_owned(),
        }),
        (SupplementScopeArg::RustcThirFixtureSpike, "branch" | "branch-control-flow" | "clamp") => {
            Ok(SupplementFixtureSelection {
                path: PathBuf::from("cargo-octet/tests/fixtures/thir-spike/branch_control_flow.rs"),
                function: "choose_branch".to_owned(),
            })
        }
        (SupplementScopeArg::RustcThirFixtureSpike, other) => Err(format!(
            "unsupported THIR supplement fixture shape `{other}`; expected `checked-arithmetic`, `call-return`, or `branch-control-flow`"
        )),
        (SupplementScopeArg::RustcMirFixtureSpike, "checked-arithmetic") => Ok(SupplementFixtureSelection {
            path: PathBuf::from("cargo-octet/tests/fixtures/mir-spike/checked.rs"),
            function: "checked".to_owned(),
        }),
        (SupplementScopeArg::RustcMirFixtureSpike, "call-return") => Ok(SupplementFixtureSelection {
            path: PathBuf::from("cargo-octet/tests/fixtures/mir-spike/call_return.rs"),
            function: "call_then_return".to_owned(),
        }),
        (SupplementScopeArg::RustcMirFixtureSpike, "branch" | "branch-control-flow" | "clamp") => {
            Ok(SupplementFixtureSelection {
                path: PathBuf::from("cargo-octet/tests/fixtures/mir-spike/branch_control_flow.rs"),
                function: "choose_branch".to_owned(),
            })
        }
        (SupplementScopeArg::RustcMirFixtureSpike, other) => Err(format!(
            "unsupported MIR supplement fixture shape `{other}`; expected `checked-arithmetic`, `call-return`, or `branch-control-flow`"
        )),
        (_, other) => Err(format!(
            "unsupported supplement fixture shape `{other}`; expected `checked-arithmetic` or `branch-control-flow`"
        )),
    }
}

fn optional_arg(args: &[String], flag: &str) -> Option<String> {
    if let Some(value) = args.iter().find_map(|arg| arg.strip_prefix(&format!("{flag}="))) {
        return Some(value.to_owned());
    }
    let index = args.iter().position(|arg| arg == flag)?;
    args.get(index + 1).cloned()
}

fn required_arg(args: &[String], flag: &str) -> Result<String, String> {
    if let Some(value) = args.iter().find_map(|arg| arg.strip_prefix(&format!("{flag}="))) {
        return Ok(value.to_owned());
    }
    let index = args.iter().position(|arg| arg == flag).ok_or_else(|| format!("missing `{flag}`"))?;
    args.get(index + 1).cloned().ok_or_else(|| format!("missing value after `{flag}`"))
}

fn required_arg_at(args: &[String], index: usize, flag: &str) -> Result<String, String> {
    let value_index = index.checked_add(1).ok_or_else(|| format!("argument index overflow while parsing `{flag}`"))?;
    args.get(value_index).cloned().ok_or_else(|| format!("missing value after `{flag}`"))
}

fn parse_prove(args: &[String], command_line: String) -> Result<ProveArgs, String> {
    let mut registry = None;
    let mut policy_artifacts_dir = None;
    let mut source = Vec::new();
    let mut family = Vec::new();
    let mut artifact_dir = None;
    let mut output_format = OutputFormat::Human;
    let mut list = false;
    let mut explain = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--registry" => {
                registry = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--policy-artifacts-dir" => {
                policy_artifacts_dir = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--source" => {
                source.push(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--family" => {
                family.push(required_arg_at(args, index, arg)?);
                index += 2;
            }
            "--artifact-dir" => {
                artifact_dir = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--output-format" | "--format" => {
                output_format = parse_output_format(&required_arg_at(args, index, arg)?)?;
                index += 2;
            }
            "--list" => {
                list = true;
                index += 1;
            }
            "--explain" => {
                explain = true;
                index += 1;
            }
            _ if arg.starts_with("--registry=") => {
                registry = Some(PathBuf::from(arg.trim_start_matches("--registry=")));
                index += 1;
            }
            _ if arg.starts_with("--policy-artifacts-dir=") => {
                policy_artifacts_dir = Some(PathBuf::from(arg.trim_start_matches("--policy-artifacts-dir=")));
                index += 1;
            }
            _ if arg.starts_with("--source=") => {
                source.push(PathBuf::from(arg.trim_start_matches("--source=")));
                index += 1;
            }
            _ if arg.starts_with("--family=") => {
                family.push(arg.trim_start_matches("--family=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--artifact-dir=") => {
                artifact_dir = Some(PathBuf::from(arg.trim_start_matches("--artifact-dir=")));
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with("--format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--format="))?;
                index += 1;
            }
            _ => return Err(format!("unsupported prove argument `{arg}`")),
        }
    }

    if registry.is_some() == policy_artifacts_dir.is_some() {
        return Err("prove requires exactly one of `--registry PATH` or `--policy-artifacts-dir PATH`".to_owned());
    }
    if source.is_empty() {
        return Err("prove requires at least one `--source PATH`".to_owned());
    }
    if output_format == OutputFormat::Sarif {
        return Err("prove does not support SARIF output; use human or json".to_owned());
    }

    Ok(ProveArgs {
        registry,
        policy_artifacts_dir,
        source,
        family,
        artifact_dir: artifact_dir.unwrap_or_else(|| PathBuf::from("artifacts/octet")),
        output_format,
        command_line,
        list,
        explain,
    })
}

fn parse_provider_manifest(args: &[String]) -> Result<ProviderManifestArgs, String> {
    match args.first().map(String::as_str) {
        Some("export") => parse_provider_manifest_export(&args[1..]),
        Some("validate") => parse_provider_manifest_validate(&args[1..]),
        Some(other) => Err(format!("unsupported provider-manifest subcommand `{other}`")),
        None => Err("missing provider-manifest subcommand".to_owned()),
    }
}

fn parse_provider_manifest_export(args: &[String]) -> Result<ProviderManifestArgs, String> {
    let mut driver = None;
    let mut lint_library = None;
    let mut config = None;
    let mut standards_policy = None;
    let mut rustc = None;
    let mut profile = None;
    let mut required_lints = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--driver" => {
                driver = Some(PathBuf::from(required_arg_at(args, index, "--driver")?));
                index += 2;
            }
            "--lint-library" => {
                lint_library = Some(PathBuf::from(required_arg_at(args, index, "--lint-library")?));
                index += 2;
            }
            "--config" => {
                config = Some(PathBuf::from(required_arg_at(args, index, "--config")?));
                index += 2;
            }
            "--standards-policy" => {
                standards_policy = Some(PathBuf::from(required_arg_at(args, index, "--standards-policy")?));
                index += 2;
            }
            "--rustc" => {
                rustc = Some(PathBuf::from(required_arg_at(args, index, "--rustc")?));
                index += 2;
            }
            "--profile" => {
                profile = Some(required_arg_at(args, index, "--profile")?);
                index += 2;
            }
            "--required-lint" => {
                required_lints.push(required_arg_at(args, index, "--required-lint")?);
                index += 2;
            }
            _ if arg.starts_with("--driver=") => {
                driver = Some(PathBuf::from(arg.trim_start_matches("--driver=")));
                index += 1;
            }
            _ if arg.starts_with("--lint-library=") => {
                lint_library = Some(PathBuf::from(arg.trim_start_matches("--lint-library=")));
                index += 1;
            }
            _ if arg.starts_with("--config=") => {
                config = Some(PathBuf::from(arg.trim_start_matches("--config=")));
                index += 1;
            }
            _ if arg.starts_with("--standards-policy=") => {
                standards_policy = Some(PathBuf::from(arg.trim_start_matches("--standards-policy=")));
                index += 1;
            }
            _ if arg.starts_with("--rustc=") => {
                rustc = Some(PathBuf::from(arg.trim_start_matches("--rustc=")));
                index += 1;
            }
            _ if arg.starts_with("--profile=") => {
                profile = Some(arg.trim_start_matches("--profile=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--required-lint=") => {
                required_lints.push(arg.trim_start_matches("--required-lint=").to_owned());
                index += 1;
            }
            _ => return Err(format!("unsupported provider-manifest export argument `{arg}`")),
        }
    }
    Ok(ProviderManifestArgs {
        mode: ProviderManifestMode::Export {
            driver: driver.ok_or_else(|| "missing `--driver`".to_owned())?,
            lint_library: lint_library.ok_or_else(|| "missing `--lint-library`".to_owned())?,
            config: config.ok_or_else(|| "missing `--config`".to_owned())?,
            standards_policy,
            rustc,
            profile,
            required_lints,
        },
    })
}

fn parse_provider_manifest_validate(args: &[String]) -> Result<ProviderManifestArgs, String> {
    let mut manifest = None;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--manifest" {
            manifest = Some(PathBuf::from(required_arg_at(args, index, "--manifest")?));
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--manifest=") {
            manifest = Some(PathBuf::from(value));
            index += 1;
        } else {
            return Err(format!("unsupported provider-manifest validate argument `{arg}`"));
        }
    }
    Ok(ProviderManifestArgs {
        mode: ProviderManifestMode::Validate {
            manifest: manifest.ok_or_else(|| "missing `--manifest`".to_owned())?,
        },
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
    use std::path::Path;

    use super::CommandArgs;
    use super::FingerprintMode;
    use super::HelpTopic;
    use super::OutputFormat;
    use super::format_command_line;
    use super::parse;

    const EXPECTED_BINARY_DECLARATION_COUNT: usize = 1;

    #[test]
    fn octet_identity_surfaces() {
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let manifest = std::fs::read_to_string(&manifest_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", manifest_path.display()));

        assert!(manifest.contains("[package]\nname = \"cargo-octet\""));
        assert_eq!(
            manifest.matches("[[bin]]\nname = \"cargo-octet\"\npath = \"src/main.rs\"").count(),
            EXPECTED_BINARY_DECLARATION_COUNT,
            "canonical cargo-octet binary must be declared exactly once"
        );
        assert_eq!(
            manifest.matches("[[bin]]\nname = \"cargo-slotcar\"\npath = \"src/main.rs\"").count(),
            EXPECTED_BINARY_DECLARATION_COUNT,
            "legacy cargo-slotcar binary remains one explicit compatibility alias"
        );
        assert_eq!(
            manifest.matches("[[bin]]\nname = \"cargo-tigerstyle\"\npath = \"src/main.rs\"").count(),
            EXPECTED_BINARY_DECLARATION_COUNT,
            "Tiger Style binary remains one explicit compatibility alias"
        );
        assert!(!manifest.contains("[package]\nname = \"cargo-slotcar\""));

        let canonical = parse(["cargo", "octet", "check", "--output-format", "json"].map(OsString::from))
            .expect("parse canonical check args");
        let alias = parse(["cargo", "slotcar", "check", "--output-format", "json"].map(OsString::from))
            .expect("parse compatibility check args");
        match (canonical, alias) {
            (CommandArgs::Check(canonical_args), CommandArgs::Check(alias_args)) => {
                assert_eq!(canonical_args.command_line, "cargo octet check --output-format json");
                assert_eq!(alias_args.command_line, canonical_args.command_line);
            }
            other => panic!("expected check args, got {other:?}"),
        }
    }

    #[test]
    fn octet_config_and_command_paths() {
        let parsed = parse(["cargo", "octet", "check", "--", "--all-targets"].map(OsString::from))
            .expect("parse canonical check command");
        match parsed {
            CommandArgs::Check(args) => {
                assert_eq!(args.command_line, "cargo octet check -- --all-targets");
                assert_eq!(args.cargo_check_args.as_deref(), Some(["--all-targets".to_owned()].as_slice()));
            }
            other => panic!("expected check args, got {other:?}"),
        }

        let error = parse(["cargo", "octet", "unknown"].map(OsString::from)).expect_err("unknown command should fail");
        assert!(error.contains("unknown subcommand"));
    }

    #[test]
    fn artifact_verify_parse_accepts_dir_and_json_report() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "artifact",
                "verify",
                "--artifact-dir",
                "artifacts/octet",
                "--output-format",
                "json",
            ]
            .map(OsString::from),
        )
        .expect("parse artifact verify args");

        match parsed {
            CommandArgs::Artifact(args) => {
                assert_eq!(args.artifact_dir, Path::new("artifacts/octet"));
                assert_eq!(args.output_format, OutputFormat::Json);
            }
            other => panic!("expected artifact args, got {other:?}"),
        }

        let missing_dir = parse(["cargo", "octet", "artifact", "verify"].map(OsString::from))
            .expect_err("artifact verify requires artifact dir");
        assert!(missing_dir.contains("artifact verify requires `--artifact-dir PATH`"));
    }

    #[test]
    fn fingerprint_parse_accepts_default_check() {
        let parsed =
            parse(["cargo", "octet", "fingerprint", "src/lib.rs"].map(OsString::from)).expect("parse fingerprint args");
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
    fn object_parse_accepts_emit_selection() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "object",
                "emit",
                "--function",
                "checked",
                "src/lib.rs",
                "--output-format",
                "json",
            ]
            .map(OsString::from),
        )
        .expect("parse object emit args");
        match parsed {
            CommandArgs::Object(args) => {
                assert_eq!(args.paths.len(), 1);
                assert_eq!(args.function.as_deref(), Some("checked"));
                assert_eq!(args.output_format, OutputFormat::Json);
            }
            other => panic!("expected object args, got {other:?}"),
        }
    }

    #[test]
    fn provider_manifest_parse_accepts_export() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "provider-manifest",
                "export",
                "--driver",
                "driver",
                "--lint-library=liboctet.so",
                "--config",
                "dylint.toml",
                "--standards-policy",
                "octet-policies.json",
                "--required-lint",
                "no_unwrap",
            ]
            .map(OsString::from),
        )
        .expect("parse provider manifest export args");
        match parsed {
            CommandArgs::ProviderManifest(args) => match args.mode {
                super::ProviderManifestMode::Export {
                    driver,
                    lint_library,
                    config,
                    standards_policy,
                    required_lints,
                    ..
                } => {
                    assert_eq!(driver, std::path::PathBuf::from("driver"));
                    assert_eq!(lint_library, std::path::PathBuf::from("liboctet.so"));
                    assert_eq!(config, std::path::PathBuf::from("dylint.toml"));
                    assert_eq!(standards_policy, Some(std::path::PathBuf::from("octet-policies.json")));
                    assert_eq!(required_lints, vec!["no_unwrap".to_owned()]);
                }
                super::ProviderManifestMode::Validate { .. } => {
                    panic!("expected provider manifest export mode")
                }
            },
            other => panic!("expected provider manifest args, got {other:?}"),
        }
    }

    #[test]
    fn provider_manifest_parse_rejects_missing_driver() {
        let error = parse(
            [
                "cargo",
                "octet",
                "provider-manifest",
                "export",
                "--lint-library",
                "liboctet.so",
                "--config",
                "dylint.toml",
            ]
            .map(OsString::from),
        )
        .expect_err("missing driver should fail");
        assert_eq!(error, "missing `--driver`");
    }

    #[test]
    fn object_parse_accepts_corpus_receipt() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "object",
                "corpus",
                "receipt",
                "--output",
                "receipt.json",
                "cargo-octet/src/object.rs",
            ]
            .map(OsString::from),
        )
        .expect("parse object corpus receipt args");
        match parsed {
            CommandArgs::Object(args) => {
                assert_eq!(args.paths.len(), 1);
                assert_eq!(args.output_format, OutputFormat::Json);
                match args.mode {
                    super::ObjectMode::CorpusReceipt { output } => {
                        assert_eq!(output, std::path::PathBuf::from("receipt.json"));
                    }
                    other => panic!("expected corpus receipt mode, got {other:?}"),
                }
            }
            other => panic!("expected object args, got {other:?}"),
        }
    }

    #[test]
    fn object_parse_accepts_corpus_contract() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "object",
                "corpus",
                "contract",
                "--output-format",
                "json",
            ]
            .map(OsString::from),
        )
        .expect("parse object corpus contract args");
        match parsed {
            CommandArgs::Object(args) => {
                assert!(matches!(args.mode, super::ObjectMode::CorpusContract));
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(args.paths.is_empty());
            }
            other => panic!("expected object args, got {other:?}"),
        }
    }

    #[test]
    fn object_parse_accepts_corpus_caveats() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "object",
                "corpus",
                "caveats",
                "--output-format",
                "json",
                "cargo-octet/src/object.rs",
            ]
            .map(OsString::from),
        )
        .expect("parse object corpus caveats args");
        match parsed {
            CommandArgs::Object(args) => {
                assert!(matches!(args.mode, super::ObjectMode::CorpusCaveats));
                assert_eq!(args.output_format, OutputFormat::Json);
                assert_eq!(args.paths.len(), 1);
            }
            other => panic!("expected object args, got {other:?}"),
        }
    }

    #[test]
    fn object_parse_accepts_corpus_check() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "object",
                "corpus",
                "check",
                "--receipt",
                "receipt.json",
                "--output-format",
                "json",
                "cargo-octet/src/object.rs",
            ]
            .map(OsString::from),
        )
        .expect("parse object corpus check args");
        match parsed {
            CommandArgs::Object(args) => {
                assert_eq!(args.paths.len(), 1);
                assert_eq!(args.output_format, OutputFormat::Json);
                match args.mode {
                    super::ObjectMode::CorpusCheck { receipt } => {
                        assert_eq!(receipt, std::path::PathBuf::from("receipt.json"));
                    }
                    other => panic!("expected corpus check mode, got {other:?}"),
                }
            }
            other => panic!("expected object args, got {other:?}"),
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn object_parse_accepts_supplement_emit_and_check() {
        let emit = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family",
                "hir",
                "--function",
                "checked",
                "src/lib.rs",
                "--output-format",
                "json",
            ]
            .map(OsString::from),
        )
        .expect("parse supplement emit args");
        match emit {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("checked"));
                assert_eq!(args.paths.len(), 1);
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Hir,
                    scope: super::SupplementScopeArg::Pilot,
                }));
            }
            other => panic!("expected object supplement emit args, got {other:?}"),
        }

        let check = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "check",
                "--family=mir",
                "--receipt",
                "supplement.json",
                "src/lib.rs",
            ]
            .map(OsString::from),
        )
        .expect("parse supplement check args");
        match check {
            CommandArgs::Object(args) => match args.mode {
                super::ObjectMode::SupplementCheck { family, scope, receipt } => {
                    assert_eq!(family, super::SupplementFamilyArg::Mir);
                    assert_eq!(scope, super::SupplementScopeArg::Pilot);
                    assert_eq!(receipt, std::path::PathBuf::from("supplement.json"));
                }
                other => panic!("expected supplement check mode, got {other:?}"),
            },
            other => panic!("expected object supplement check args, got {other:?}"),
        }

        let thir = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--function",
                "checked",
                "src/lib.rs",
            ]
            .map(OsString::from),
        )
        .expect("parse THIR supplement emit args");
        match thir {
            CommandArgs::Object(args) => match args.mode {
                super::ObjectMode::SupplementEmit { family, scope } => {
                    assert_eq!(family, super::SupplementFamilyArg::Thir);
                    assert_eq!(scope, super::SupplementScopeArg::Pilot);
                }
                other => panic!("expected THIR supplement emit mode, got {other:?}"),
            },
            other => panic!("expected object THIR supplement args, got {other:?}"),
        }

        let spike = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=hir",
                "--spike=rustc-hir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse rustc HIR spike supplement args");
        match spike {
            CommandArgs::Object(args) => match args.mode {
                super::ObjectMode::SupplementEmit { family, scope } => {
                    assert_eq!(family, super::SupplementFamilyArg::Hir);
                    assert_eq!(scope, super::SupplementScopeArg::RustcHirFixtureSpike);
                }
                other => panic!("expected supplement spike emit mode, got {other:?}"),
            },
            other => panic!("expected object supplement spike args, got {other:?}"),
        }

        let branch_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=hir",
                "--spike=rustc-hir-fixture",
                "--fixture-shape=branch-control-flow",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse branch fixture shape supplement args");
        match branch_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("clamp_to_limit"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/hir-spike/clamp.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Hir,
                    scope: super::SupplementScopeArg::RustcHirFixtureSpike,
                }));
            }
            other => panic!("expected branch fixture shape args, got {other:?}"),
        }

        let shape_without_spike = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=hir",
                "--fixture-shape=branch-control-flow",
            ]
            .map(OsString::from),
        )
        .expect_err("fixture shape requires rustc spike scope");
        assert!(shape_without_spike.contains(
            "requires `--spike rustc-hir-fixture`, `--spike rustc-thir-fixture`, or `--spike rustc-mir-fixture`"
        ));

        let thir_without_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect_err("THIR fixture spike requires explicit fixture shape");
        assert!(
            thir_without_shape
                .contains("requires explicit `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`")
        );

        let thir_unknown_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--fixture-shape=not-selected",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect_err("THIR fixture spike rejects non-selected fixture shape");
        assert!(
            thir_unknown_shape.contains(
                "unsupported THIR supplement fixture shape `not-selected`; expected `checked-arithmetic`, `call-return`, or `branch-control-flow`"
            ),
            "unexpected THIR unknown fixture shape error: {thir_unknown_shape}"
        );

        let thir_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--fixture-shape=checked-arithmetic",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse THIR fixture shape supplement args");
        match thir_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("checked"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/thir-spike/checked.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Thir,
                    scope: super::SupplementScopeArg::RustcThirFixtureSpike,
                }));
            }
            other => panic!("expected THIR fixture shape args, got {other:?}"),
        }

        let thir_call_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--fixture-shape=call-return",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse THIR call-return fixture shape supplement args");
        match thir_call_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("call_then_return"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/thir-spike/call_return.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Thir,
                    scope: super::SupplementScopeArg::RustcThirFixtureSpike,
                }));
            }
            other => panic!("expected THIR call-return fixture shape args, got {other:?}"),
        }

        let thir_branch_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--fixture-shape=branch-control-flow",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse THIR branch-control-flow fixture shape supplement args");
        match thir_branch_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("choose_branch"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/thir-spike/branch_control_flow.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Thir,
                    scope: super::SupplementScopeArg::RustcThirFixtureSpike,
                }));
            }
            other => panic!("expected THIR branch-control-flow fixture shape args, got {other:?}"),
        }
        let mir_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--fixture-shape=checked-arithmetic",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse MIR fixture shape supplement args");
        match mir_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("checked"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/mir-spike/checked.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Mir,
                    scope: super::SupplementScopeArg::RustcMirFixtureSpike,
                }));
            }
            other => panic!("expected MIR fixture shape args, got {other:?}"),
        }

        let mir_without_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect_err("MIR fixture spike requires explicit fixture shape");
        assert!(
            mir_without_shape
                .contains("requires explicit `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`")
        );

        let mir_checked_alias = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--fixture-shape=checked",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect_err("MIR fixture spike rejects aliases outside inventory selector");
        assert!(
            mir_checked_alias
                .contains("supports only `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`"),
            "unexpected MIR checked alias fixture shape error: {mir_checked_alias}"
        );

        let mir_call_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--fixture-shape=call-return",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse MIR call-return fixture shape supplement args");
        match mir_call_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("call_then_return"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/mir-spike/call_return.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Mir,
                    scope: super::SupplementScopeArg::RustcMirFixtureSpike,
                }));
            }
            other => panic!("expected MIR call-return fixture shape args, got {other:?}"),
        }

        let mir_branch_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--fixture-shape=branch-control-flow",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse MIR branch-control-flow fixture shape supplement args");
        match mir_branch_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("choose_branch"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/mir-spike/branch_control_flow.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Mir,
                    scope: super::SupplementScopeArg::RustcMirFixtureSpike,
                }));
            }
            other => panic!("expected MIR branch-control-flow fixture shape args, got {other:?}"),
        }
    }

    #[test]
    fn object_parse_accepts_supplement_fixture_inventory() {
        let inventory = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "fixtures",
                "--family=hir",
                "--spike=rustc-hir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse supplement fixture inventory args");
        match inventory {
            CommandArgs::Object(args) => {
                assert_eq!(args.paths, Vec::<std::path::PathBuf>::new());
                assert_eq!(args.function, None);
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(matches!(args.mode, super::ObjectMode::SupplementFixtures {
                    family: super::SupplementFamilyArg::Hir,
                    scope: super::SupplementScopeArg::RustcHirFixtureSpike,
                }));
            }
            other => panic!("expected object supplement fixtures args, got {other:?}"),
        }

        let missing_scope =
            parse(["cargo", "octet", "object", "supplement", "fixtures", "--family=hir"].map(OsString::from))
                .expect_err("fixture inventory requires rustc spike scope");
        assert!(missing_scope.contains("requires `--spike rustc-hir-fixture`"));

        let thir_inventory = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "fixtures",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse THIR supplement fixture inventory args");
        match thir_inventory {
            CommandArgs::Object(args) => {
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(matches!(args.mode, super::ObjectMode::SupplementFixtures {
                    family: super::SupplementFamilyArg::Thir,
                    scope: super::SupplementScopeArg::RustcThirFixtureSpike,
                }));
            }
            other => panic!("expected THIR object supplement fixtures args, got {other:?}"),
        }

        let mir_inventory = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "fixtures",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse MIR supplement fixture inventory args");
        match mir_inventory {
            CommandArgs::Object(args) => {
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(matches!(args.mode, super::ObjectMode::SupplementFixtures {
                    family: super::SupplementFamilyArg::Mir,
                    scope: super::SupplementScopeArg::RustcMirFixtureSpike,
                }));
            }
            other => panic!("expected MIR object supplement fixtures args, got {other:?}"),
        }

        let mir_wrong_scope = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "fixtures",
                "--family=mir",
                "--spike=rustc-hir-fixture",
            ]
            .map(OsString::from),
        )
        .expect_err("MIR fixture inventory rejects HIR scope");
        assert!(mir_wrong_scope.contains("requires `--spike rustc-mir-fixture`"));
    }

    #[test]
    fn help_parse_accepts_root_and_subcommand_help() {
        let root = parse(["cargo", "octet", "--help"].map(OsString::from)).expect("parse root help");
        assert!(matches!(root, CommandArgs::Help(HelpTopic::Root)));

        let artifact =
            parse(["cargo", "octet", "artifact", "verify", "--help"].map(OsString::from)).expect("parse artifact help");
        assert!(matches!(artifact, CommandArgs::Help(HelpTopic::Artifact)));

        let check = parse(["cargo", "octet", "check", "--help"].map(OsString::from)).expect("parse check help");
        assert!(matches!(check, CommandArgs::Help(HelpTopic::Check)));

        for args in [
            &["cargo", "octet", "object", "--help"][..],
            &["cargo", "octet", "object", "supplement", "fixtures", "--help"][..],
            &["cargo", "octet", "object", "supplement", "emit", "--help"][..],
            &["cargo", "octet", "object", "supplement", "check", "--help"][..],
        ] {
            let parsed = parse(args.iter().map(OsString::from)).expect("parse object help");
            assert!(matches!(parsed, CommandArgs::Help(HelpTopic::Object)));
        }
    }

    #[test]
    fn version_parse_accepts_root_version_aliases() {
        for args in [
            ["cargo", "octet", "--version"],
            ["cargo", "octet", "-V"],
            ["cargo", "octet", "version"],
            ["cargo", "tigerstyle", "--version"],
        ] {
            let parsed = parse(args.map(OsString::from)).expect("parse version");
            assert!(matches!(parsed, CommandArgs::Version));
        }
    }

    #[test]
    fn command_line_quoting_matches_contract() {
        assert_eq!(format_command_line("cargo", &["octet".to_owned(), "check".to_owned()]), "cargo octet check");
        assert_eq!(format_command_line("cargo", &["arg with spaces".to_owned()]), "cargo 'arg with spaces'");
        assert_eq!(format_command_line("cargo", &["can't".to_owned()]), "cargo 'can'\"'\"'t'");
        assert_eq!(format_command_line("cargo", &[String::new()]), "cargo ''");
    }

    #[test]
    fn compatibility_alias_command_line_records_octet_canonical_identity() {
        let parsed = parse(["cargo", "slotcar", "check", "--output-format", "json"].map(OsString::from))
            .expect("parse alias check args");
        match parsed {
            CommandArgs::Check(args) => assert_eq!(args.command_line, "cargo octet check --output-format json"),
            other => panic!("expected check args, got {other:?}"),
        }
    }
}
