pub(crate) mod ast;
pub(crate) mod closure;
#[cfg(test)]
pub(crate) mod contract;
pub(crate) mod policy;
pub(crate) mod scan;
#[cfg(test)]
mod tests;

use serde::Deserialize;
use serde::Serialize;

const BLAKE3_PREFIX: &str = "b3:";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BindingProfile {
    ImplementationBoundV1,
    LockfileOnlyV1,
}

impl BindingProfile {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ImplementationBoundV1 => "implementation-bound-v1",
            Self::LockfileOnlyV1 => "lockfile-only-v1",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SummaryJustification {
    CheckedProof,
    ReviewedAssumption,
    UnresolvedObligation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    LocalSourceTree,
    LockedRegistryOrGit,
    Toolchain,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompletenessStatus {
    Complete,
    Incomplete,
    UnsupportedGeneratedInput,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct GeneratedInputBinding {
    pub locator: String,
    pub blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ImplementationBinding {
    pub profile: BindingProfile,
    pub package_envelope_blake3: String,
    pub closure_blake3: String,
    pub build_context_blake3: String,
    pub justification: SummaryJustification,
    #[serde(default = "empty_generated_inputs", skip_serializing_if = "Vec::is_empty")]
    pub generated_inputs: Vec<GeneratedInputBinding>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProviderRecord {
    pub provider: crate::effect_closure::ProviderIdentity,
    pub source_envelope_kind: SourceKind,
    pub completeness: CompletenessStatus,
    #[serde(default = "empty_strings", skip_serializing_if = "Vec::is_empty")]
    pub completeness_diagnostics: Vec<String>,
    pub package_envelope_blake3: String,
    pub build_context_blake3: String,
    pub implementation_closure_blake3: String,
    #[serde(default = "empty_generated_inputs", skip_serializing_if = "Vec::is_empty")]
    pub generated_inputs: Vec<GeneratedInputBinding>,
    #[serde(default = "empty_providers", skip_serializing_if = "Vec::is_empty")]
    pub transitive_providers: Vec<crate::effect_closure::ProviderIdentity>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct JustificationSummary {
    pub reviewed_assumptions: usize,
    pub checked_proofs: usize,
    pub unresolved_obligations: usize,
    pub formal_proof_promoted: bool,
}

fn empty_generated_inputs() -> Vec<GeneratedInputBinding> {
    Vec::new()
}

fn empty_strings() -> Vec<String> {
    Vec::new()
}

fn empty_providers() -> Vec<crate::effect_closure::ProviderIdentity> {
    Vec::new()
}

pub(crate) fn read_regular_non_symlink_file(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| format!("inspect `{}`: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("path `{}` is a symlink and is rejected", path.display()));
    }
    if !metadata.file_type().is_file() {
        return Err(format!("path `{}` is not a regular file", path.display()));
    }
    std::fs::read(path).map_err(|error| format!("read `{}`: {error}", path.display()))
}

pub(crate) fn framed_blake3(domain: &str, parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    for part in parts {
        if let Ok(len) = u64::try_from(part.len()) {
            hasher.update(&len.to_le_bytes());
        }
        hasher.update(part);
    }
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedDepEdge {
    pub(crate) pkg_id: String,
    pub(crate) dep_kinds: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedPackageInput {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: Option<String>,
    pub(crate) manifest_path: std::path::PathBuf,
    pub(crate) is_workspace_member: bool,
    pub(crate) features: Vec<String>,
    pub(crate) targets: Vec<scan::PackageTargetMeta>,
    pub(crate) deps: Vec<ResolvedDepEdge>,
}

#[derive(Clone, Debug)]
pub(crate) struct ClosureMetadataInput {
    pub(crate) target: String,
    pub(crate) toolchain: String,
    pub(crate) cargo_lock_blake3: String,
    pub(crate) build_context: policy::BuildContextOutcome,
    pub(crate) has_resolve_graph: bool,
    pub(crate) packages: Vec<ResolvedPackageInput>,
    pub(crate) toolchain_provider_names: Vec<String>,
}

pub(crate) fn merge_completeness(left: CompletenessStatus, right: CompletenessStatus) -> CompletenessStatus {
    match (left, right) {
        (CompletenessStatus::Incomplete, _) | (_, CompletenessStatus::Incomplete) => CompletenessStatus::Incomplete,
        (CompletenessStatus::UnsupportedGeneratedInput, _) | (_, CompletenessStatus::UnsupportedGeneratedInput) => {
            CompletenessStatus::UnsupportedGeneratedInput
        }
        (CompletenessStatus::Complete, CompletenessStatus::Complete) => CompletenessStatus::Complete,
    }
}

pub(crate) fn relative_locator(root: &std::path::Path, path: &std::path::Path) -> Result<String, String> {
    let rel = path.strip_prefix(root).map_err(|_| format!("path `{}` escaped package root", path.display()))?;
    let parts = rel
        .components()
        .map(|comp| {
            let std::path::Component::Normal(seg) = comp else {
                return Err(format!("invalid relative component in `{}`", rel.display()));
            };
            seg.to_str().ok_or_else(|| format!("non-UTF-8 relative path `{}`", rel.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parts.join("/"))
}

pub(crate) fn canonical_blake3_identity(value: &str) -> String {
    if value.starts_with(BLAKE3_PREFIX) {
        value.to_owned()
    } else {
        format!("{BLAKE3_PREFIX}{value}")
    }
}

pub(crate) fn format_package_source(
    is_workspace_member: bool,
    source: Option<&str>,
    completeness: CompletenessStatus,
    closure_blake3: &str,
) -> String {
    if is_workspace_member {
        if completeness == CompletenessStatus::Complete {
            format!("workspace-envelope:{closure_blake3}")
        } else {
            format!("workspace-envelope:incomplete:{closure_blake3}")
        }
    } else if let Some(source) = source {
        source.to_owned()
    } else if completeness == CompletenessStatus::Complete {
        format!("path-envelope:{closure_blake3}")
    } else {
        format!("path-envelope:incomplete:{closure_blake3}")
    }
}

pub(crate) fn append_toolchain_envelopes(
    target: &str,
    toolchain: &str,
    build_context: &policy::BuildContextOutcome,
    names: &[String],
    records: &mut Vec<ProviderRecord>,
) {
    let completeness = if build_context.incomplete_reason.is_some() {
        CompletenessStatus::Incomplete
    } else {
        CompletenessStatus::Complete
    };
    let diagnostics = build_context.incomplete_reason.iter().cloned().collect::<Vec<_>>();
    for name in names {
        let provider = crate::effect_closure::ProviderIdentity {
            kind: crate::effect_closure::ProviderKind::Toolchain,
            name: name.clone(),
            version: toolchain.to_owned(),
            source: "rustc-toolchain".to_owned(),
            target: target.to_owned(),
            features: Vec::new(),
        };
        let pkg_env = crate::effect_closure::provider_identity_blake3(&provider);
        let closure = framed_blake3("octet-effect-closure-toolchain-closure/v1", &[
            pkg_env.as_bytes(),
            build_context.blake3.as_bytes(),
        ]);
        records.push(ProviderRecord {
            provider,
            source_envelope_kind: SourceKind::Toolchain,
            completeness,
            completeness_diagnostics: diagnostics.clone(),
            package_envelope_blake3: pkg_env,
            build_context_blake3: build_context.blake3.clone(),
            implementation_closure_blake3: closure,
            generated_inputs: Vec::new(),
            transitive_providers: Vec::new(),
        });
    }
}
