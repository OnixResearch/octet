//! Bounded Cairn `cairn.content-bound-trace-index.v2` parser and domain-identity verifier.

const CAIRN_INDEX_SCHEMA_V2: &str = "cairn.content-bound-trace-index.v2";
const CAIRN_INDEX_DOMAIN_V2: &str = "cairn.trace-index.structural-index.v2";
const CAIRN_BINDING_DOMAIN_V1: &str = "cairn.trace-index.rust-item-binding.v1";
const CAIRN_ADAPTER_V1: &str = "cairn.saved-rust-bytes.v1";
const CAIRN_PARSER_SYN: &str = "syn-2.0.117";
const CAIRN_BINDING_SCHEMA_V1: &str = "cairn.rust-item-binding.v1";

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct TraceSourceIdentity {
    path: String,
    content_blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct TraceLink {
    requirement_id: String,
    requirement_row_blake3: String,
    role: String,
    source_path: String,
    source_file_blake3: String,
    marker_span: super::AnchorSpan,
    marker_blake3: String,
    link_blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct RawRustBinding {
    link_blake3: String,
    requirement_id: String,
    requirement_row_blake3: String,
    source_path: String,
    source_file_blake3: String,
    marker_span: super::AnchorSpan,
    item: super::ParsedAnchor,
    configuration: super::ItemConfiguration,
    binding_blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct RawTraceData {
    adapter: String,
    parser: String,
    schema: String,
    configuration: super::ItemConfiguration,
    bindings: Vec<RawRustBinding>,
    history: Vec<RawRustBinding>,
}

/// Converts a raw `cairn.content-bound-trace-index.v2` JSON payload without executing external
/// tools.
pub(crate) fn adapt_report(
    profile: &super::IntakeProfile,
    report_bytes: &[u8],
) -> Result<super::AdapterOutput, String> {
    let mut root: serde_json::Value =
        serde_json::from_slice(report_bytes).map_err(|error| format!("parse Cairn report JSON: {error}"))?;
    let report_schema = root.get("schema_version").and_then(serde_json::Value::as_str).unwrap_or_default().to_owned();
    if report_schema != profile.expected_report_schema || report_schema != CAIRN_INDEX_SCHEMA_V2 {
        return Ok(super::AdapterOutput {
            report_schema: report_schema.clone(),
            records: Vec::new(),
            historical_records: Vec::new(),
            unsupported_reasons: vec![format!(
                "unsupported Cairn schema `{report_schema}`; expected `{}`",
                profile.expected_report_schema
            )],
            diagnostics: Vec::new(),
        });
    }
    let mut diagnostics = Vec::new();
    let mut unsupported_reasons = Vec::new();
    verify_index_envelope(&mut root, &mut diagnostics)?;
    let sources: Vec<TraceSourceIdentity> = parse_field(&root, "sources", &mut diagnostics)?;
    let links: Vec<TraceLink> = parse_field(&root, "links", &mut diagnostics)?;
    let trace_data: Option<RawTraceData> = match root.get("structural") {
        None | Some(serde_json::Value::Null) => None,
        Some(_) => Some(parse_field(&root, "structural", &mut diagnostics)?),
    };
    let Some(data) = trace_data else {
        unsupported_reasons.push("Cairn trace index carries no `structural` payload".to_owned());
        return Ok(super::AdapterOutput {
            report_schema,
            records: Vec::new(),
            historical_records: Vec::new(),
            unsupported_reasons,
            diagnostics,
        });
    };
    check_adapter_metadata(profile, &data, &mut unsupported_reasons, &mut diagnostics);
    let source_map = build_source_map(&sources, &mut diagnostics);
    let link_map = build_link_map(&links, &mut diagnostics);
    let records = convert_active_bindings(
        &BindingContext {
            profile,
            data: &data,
            sources: &source_map,
            links: &link_map,
        },
        &mut diagnostics,
    );
    let historical_records = data
        .history
        .iter()
        .map(|entry| super::HistoricalRecord {
            requirement_id: entry.requirement_id.clone(),
            source_path: entry.source_path.clone(),
            binding_blake3: entry.binding_blake3.clone(),
            historical_only: true,
            note: "historical Cairn binding; never admitted as current evidence".to_owned(),
        })
        .collect();
    Ok(super::AdapterOutput {
        report_schema,
        records,
        historical_records,
        unsupported_reasons,
        diagnostics,
    })
}

fn parse_field<T: serde::de::DeserializeOwned>(
    root: &serde_json::Value,
    key: &str,
    diagnostics: &mut Vec<String>,
) -> Result<T, String> {
    let field = root.get(key).ok_or_else(|| format!("Cairn report missing `{key}`"))?;
    serde_json::from_value::<T>(field.clone()).map_err(|error| {
        let message = format!("invalid Cairn report field `{key}`: {error}");
        diagnostics.push(message.clone());
        message
    })
}

fn verify_index_envelope(root: &mut serde_json::Value, diagnostics: &mut Vec<String>) -> Result<(), String> {
    let declared_hash = root
        .get("index_blake3")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "Cairn report missing `index_blake3`".to_owned())?
        .to_owned();
    if !super::is_blake3_hex(&declared_hash) {
        diagnostics.push("Cairn `index_blake3` is not a lowercase BLAKE3 hex digest".to_owned());
    }
    let object = root.as_object_mut().ok_or_else(|| "Cairn report JSON root must be an object".to_owned())?;
    let removed = object.remove("index_blake3");
    let computed = super::domain_hash_json(CAIRN_INDEX_DOMAIN_V2, object)?;
    if let Some(val) = removed {
        object.insert("index_blake3".to_owned(), val);
    }
    if computed != declared_hash {
        diagnostics.push(format!("Cairn `index_blake3` mismatch: expected {computed}, actual {declared_hash}"));
    }
    if object.get("non_claims").and_then(serde_json::Value::as_array).is_none_or(Vec::is_empty) {
        diagnostics.push("Cairn report is missing required `non_claims`".to_owned());
    }
    Ok(())
}

fn check_adapter_metadata(
    profile: &super::IntakeProfile,
    data: &RawTraceData,
    unsupported: &mut Vec<String>,
    diagnostics: &mut Vec<String>,
) {
    if data.schema != profile.expected_structural_schema || data.schema != CAIRN_BINDING_SCHEMA_V1 {
        unsupported.push(format!(
            "unsupported Cairn binding schema `{}`; expected `{}`",
            data.schema, profile.expected_structural_schema
        ));
    }
    if data.adapter != profile.expected_adapter || data.adapter != CAIRN_ADAPTER_V1 {
        unsupported
            .push(format!("unsupported Cairn adapter `{}`; expected `{}`", data.adapter, profile.expected_adapter));
    }
    if data.parser != profile.expected_parser || data.parser != CAIRN_PARSER_SYN {
        diagnostics.push(format!(
            "Cairn parser identity mismatch: expected `{}`, actual `{}`",
            profile.expected_parser, data.parser
        ));
    }
}

fn build_source_map<'a>(
    sources: &'a [TraceSourceIdentity],
    diagnostics: &mut Vec<String>,
) -> std::collections::BTreeMap<&'a str, &'a str> {
    let mut map = std::collections::BTreeMap::new();
    for src in sources {
        if !super::is_blake3_hex(&src.content_blake3) {
            diagnostics.push(format!("invalid source digest for `{}`", src.path));
        }
        if map.insert(src.path.as_str(), src.content_blake3.as_str()).is_some() {
            diagnostics.push(format!("duplicate source entry `{}` in Cairn report", src.path));
        }
    }
    map
}

fn build_link_map<'a>(
    links: &'a [TraceLink],
    diagnostics: &mut Vec<String>,
) -> std::collections::BTreeMap<&'a str, &'a TraceLink> {
    let mut map = std::collections::BTreeMap::new();
    for link in links {
        if map.insert(link.link_blake3.as_str(), link).is_some() {
            diagnostics.push(format!("duplicate `link_blake3` `{}` in Cairn report", link.link_blake3));
        }
    }
    map
}

struct BindingContext<'a> {
    profile: &'a super::IntakeProfile,
    data: &'a RawTraceData,
    sources: &'a std::collections::BTreeMap<&'a str, &'a str>,
    links: &'a std::collections::BTreeMap<&'a str, &'a TraceLink>,
}

fn convert_active_bindings(ctx: &BindingContext<'_>, diagnostics: &mut Vec<String>) -> Vec<super::LinkRecord> {
    let mut seen_bindings = std::collections::BTreeSet::new();
    let mut out = Vec::with_capacity(ctx.data.bindings.len());
    for raw in &ctx.data.bindings {
        if !seen_bindings.insert(raw.binding_blake3.as_str()) {
            diagnostics.push(format!("duplicate active binding `{}`", raw.binding_blake3));
            continue;
        }
        validate_raw_binding(ctx, raw, diagnostics);
        let relation_kind = ctx.links.get(raw.link_blake3.as_str()).map_or_else(String::new, |link| link.role.clone());
        out.push(super::LinkRecord {
            requirement_id: raw.requirement_id.clone(),
            requirement_row_blake3: raw.requirement_row_blake3.clone(),
            relation_kind,
            namespace: ctx.profile.namespace_prefix.clone(),
            producer: ctx.profile.producer.clone(),
            anchor_kind: ctx.profile.expected_anchor_kind.clone(),
            source_path: raw.source_path.clone(),
            source_file_blake3: raw.source_file_blake3.clone(),
            marker_span: raw.marker_span.clone(),
            anchor: Some(raw.item.clone()),
            configuration: Some(raw.configuration.clone()),
            link_blake3: raw.link_blake3.clone(),
            binding_blake3: raw.binding_blake3.clone(),
        });
    }
    out
}

fn validate_raw_binding(ctx: &BindingContext<'_>, raw: &RawRustBinding, diagnostics: &mut Vec<String>) {
    if raw.configuration != ctx.data.configuration {
        diagnostics.push(format!("binding `{}` config disagrees with root", raw.binding_blake3));
    }
    match ctx.links.get(raw.link_blake3.as_str()) {
        None => diagnostics.push(format!("binding `{}` missing link `{}`", raw.binding_blake3, raw.link_blake3)),
        Some(link) => {
            if (link.requirement_id != raw.requirement_id)
                || (link.requirement_row_blake3 != raw.requirement_row_blake3)
            {
                diagnostics.push(format!("binding `{}` req disagrees with link", raw.binding_blake3));
            }
            if (link.source_path != raw.source_path) || (link.source_file_blake3 != raw.source_file_blake3) {
                diagnostics.push(format!("binding `{}` source role disagrees with link", raw.binding_blake3));
            }
            if !ctx.profile.supported_relations.iter().any(|allowed| allowed == &link.role) {
                diagnostics.push(format!("binding `{}` unsupported relation `{}`", raw.binding_blake3, link.role));
            }
        }
    }
    if ctx.sources.get(raw.source_path.as_str()) != Some(&raw.source_file_blake3.as_str()) {
        diagnostics.push(format!("binding `{}` source disagrees with sources", raw.binding_blake3));
    }
    let expected = compute_binding_hash(raw).unwrap_or_default();
    if expected != raw.binding_blake3 {
        diagnostics.push(format!("binding `{}` expected {expected}, got {}", raw.requirement_id, raw.binding_blake3));
    }
}

fn compute_binding_hash(raw: &RawRustBinding) -> Result<String, String> {
    super::domain_hash_json(
        CAIRN_BINDING_DOMAIN_V1,
        &(
            raw.link_blake3.as_str(),
            raw.requirement_id.as_str(),
            raw.requirement_row_blake3.as_str(),
            raw.source_path.as_str(),
            raw.source_file_blake3.as_str(),
            &raw.marker_span,
            &raw.item,
            CAIRN_ADAPTER_V1,
            CAIRN_PARSER_SYN,
            CAIRN_BINDING_SCHEMA_V1,
            &raw.configuration,
        ),
    )
}
