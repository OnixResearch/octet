//! Pure observations over measured Aeneas provider artifacts.
//!
//! The imperative shell reads the produced LLBC, `translation.json`, generated Lean, and the
//! maintained theorem source; these functions turn those bytes into the typed facts that the
//! receipt records and the validators compare against the reviewed provider profile.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::Value;

use crate::formal_proof::AENEAS_TRANSLATION_MANIFEST_SCHEMA_VERSION;
use crate::formal_proof::AeneasAssumptionPolicy;
use crate::formal_proof::AeneasGeneratedLeanFile;
use crate::formal_proof::AeneasLlbcDeclaration;
use crate::formal_proof::AeneasSourceSpan;
use crate::formal_proof::AeneasTranslationDeclaration;
use crate::formal_proof::AeneasTranslationManifest;
use crate::formal_proof::AeneasTrustClosure;
use crate::formal_proof::CharonExtractionPolicy;
use crate::formal_proof::validate_aeneas_translation_manifest;

/// Reviewed-vocabulary MIR stage for Charon's default (`mir = null`) borrow-checked MIR.
const MIR_STAGE_DEFAULT: &str = "borrowck";
/// Reviewed-vocabulary drop policy when neither precise nor desugared drops are requested.
const DROP_POLICY_DEFAULT: &str = "include-drops";
/// Reviewed-vocabulary `start_from` when no item selection narrows the translated crate.
const START_FROM_CRATE: &str = "crate";
/// Reviewed-vocabulary transformation setting observed when every declaration keeps its span.
const TRANSFORMATION_PRESERVE_SPANS: &str = "preserve-spans";
/// Reviewed-vocabulary serialization of JSON-encoded structured LLBC.
const SERIALIZATION_LLBC_JSON: &str = "llbc-json-v1";
const JSON_FORMAT: &str = "json";
const BITS_PER_BYTE: u64 = 8;
const DECLARATION_COLLECTIONS: [&str; 5] = ["fun_decls", "type_decls", "global_decls", "trait_decls", "trait_impls"];
const THEOREM_ORIGIN: &str = "theorem";
const GENERATED_ORIGIN: &str = "generated";

/// Observe the effective Charon extraction policy from the produced LLBC bytes.
///
/// Charon's serialized options are mapped into the reviewed provider-profile vocabulary:
/// `mir = null` is the default borrow-checked MIR (`borrowck`); no precise or desugared drops is
/// `include-drops`; empty `start_from*` selections without `start_from_pub` is `crate`; every
/// declaration carrying a source span is `preserve-spans`; and JSON-encoded structured LLBC is
/// `llbc-json-v1`. Any other value is recorded in a form that cannot equal the reviewed
/// vocabulary, so the contract comparison fails closed.
///
/// # Errors
///
/// Returns diagnostics when the LLBC is not JSON or omits a field the mapping requires.
// r[impl octet.aeneas_lean_proof_rail.charon_contract]
pub fn observe_charon_extraction_policy(llbc_text: &str) -> Result<CharonExtractionPolicy, Vec<String>> {
    let value =
        serde_json::from_str::<Value>(llbc_text).map_err(|error| vec![format!("parse Charon LLBC: {error}")])?;
    let mut diagnostics = Vec::new();
    let has_errors = required_bool(&value, "/has_errors", &mut diagnostics);
    let translated = &value["translated"];
    let options = &translated["options"];
    if !options.is_object() {
        diagnostics.push("Charon LLBC is missing translated.options".to_owned());
        return Err(diagnostics);
    }
    let skip_borrowck = required_bool(options, "/skip_borrowck", &mut diagnostics);
    let (target_triple, pointer_width_bits, endianness) = observed_target(translated, &mut diagnostics);
    let mir_start_stage = match options.get("mir") {
        Some(Value::Null) => MIR_STAGE_DEFAULT.to_owned(),
        Some(Value::String(stage)) => format!("mir:{stage}"),
        _ => {
            diagnostics.push("Charon LLBC options.mir is missing or malformed".to_owned());
            String::new()
        }
    };
    let precise_drops = required_bool(options, "/precise_drops", &mut diagnostics);
    let desugar_drops = required_bool(options, "/desugar_drops", &mut diagnostics);
    let drop_policy = match (precise_drops, desugar_drops) {
        (false, false) => DROP_POLICY_DEFAULT.to_owned(),
        (precise, desugar) => format!("precise-drops={precise},desugar-drops={desugar}"),
    };
    let start_from = observed_start_from(options, &mut diagnostics);
    let include_rules = required_strings(options, "include", &mut diagnostics);
    let exclude_rules = required_strings(options, "exclude", &mut diagnostics);
    let opaque_rules = required_strings(options, "opaque", &mut diagnostics);
    let mut transformation_settings = Vec::new();
    if declarations_preserve_spans(translated, &mut diagnostics) {
        transformation_settings.push(TRANSFORMATION_PRESERVE_SPANS.to_owned());
    }
    let serialization_format = observed_serialization(options, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(CharonExtractionPolicy {
            has_errors,
            skip_borrowck,
            target_triple,
            pointer_width_bits,
            endianness,
            mir_start_stage,
            drop_policy,
            start_from,
            include_rules,
            exclude_rules,
            opaque_rules,
            transformation_settings,
            serialization_format,
        })
    } else {
        diagnostics.sort();
        diagnostics.dedup();
        Err(diagnostics)
    }
}

fn required_bool(value: &Value, pointer: &str, diagnostics: &mut Vec<String>) -> bool {
    if let Some(flag) = value.pointer(pointer).and_then(Value::as_bool) {
        flag
    } else {
        diagnostics.push(format!("Charon LLBC field `{pointer}` is missing or not a boolean"));
        // Fail toward the weakened value so a missing flag can never look reviewed.
        true
    }
}

fn required_strings(options: &Value, field: &str, diagnostics: &mut Vec<String>) -> Vec<String> {
    let Some(values) = options.get(field).and_then(Value::as_array) else {
        diagnostics.push(format!("Charon LLBC options.{field} is missing or not an array"));
        return Vec::new();
    };
    values
        .iter()
        .map(|value| {
            value.as_str().map_or_else(
                || {
                    diagnostics.push(format!("Charon LLBC options.{field} contains a non-string rule"));
                    String::new()
                },
                str::to_owned,
            )
        })
        .collect()
}

fn observed_target(translated: &Value, diagnostics: &mut Vec<String>) -> (String, u16, String) {
    let rows = translated.get("target_information").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    let [row] = rows else {
        diagnostics.push(format!("Charon LLBC must record exactly one target, found {}", rows.len()));
        return (String::new(), 0, String::new());
    };
    let triple = row.get("key").and_then(Value::as_str).unwrap_or_default().to_owned();
    if triple.is_empty() {
        diagnostics.push("Charon LLBC target triple is missing".to_owned());
    }
    let pointer_width_bits = row
        .pointer("/value/target_pointer_size")
        .and_then(Value::as_u64)
        .and_then(|bytes| bytes.checked_mul(BITS_PER_BYTE))
        .and_then(|bits| u16::try_from(bits).ok())
        .unwrap_or_else(|| {
            diagnostics.push("Charon LLBC target pointer size is missing or malformed".to_owned());
            0
        });
    let endianness = match row.pointer("/value/is_little_endian").and_then(Value::as_bool) {
        Some(true) => "little".to_owned(),
        Some(false) => "big".to_owned(),
        None => {
            diagnostics.push("Charon LLBC target endianness is missing".to_owned());
            String::new()
        }
    };
    (triple, pointer_width_bits, endianness)
}

fn observed_start_from(options: &Value, diagnostics: &mut Vec<String>) -> String {
    let mut selections = Vec::new();
    for field in ["start_from", "start_from_if_exists", "start_from_attribute"] {
        for rule in required_strings(options, field, diagnostics) {
            selections.push(format!("{field}:{rule}"));
        }
    }
    let by_visibility = required_bool(options, "/start_from_pub", diagnostics);
    if by_visibility {
        selections.push("start_from_pub".to_owned());
    }
    if selections.is_empty() {
        START_FROM_CRATE.to_owned()
    } else {
        selections.join(",")
    }
}

fn declarations_preserve_spans(translated: &Value, diagnostics: &mut Vec<String>) -> bool {
    let mut preserved = true;
    for collection in DECLARATION_COLLECTIONS {
        let Some(declarations) = translated.get(collection).and_then(Value::as_array) else {
            diagnostics.push(format!("Charon LLBC translated.{collection} is missing"));
            return false;
        };
        preserved &= declarations
            .iter()
            .filter(|declaration| !declaration.is_null())
            .all(|declaration| declaration.pointer("/item_meta/span/data").is_some_and(Value::is_object));
    }
    preserved
}

fn observed_serialization(options: &Value, diagnostics: &mut Vec<String>) -> String {
    let unstructured = required_bool(options, "/ullbc", diagnostics);
    let not_serialized = required_bool(options, "/no_serialize", diagnostics);
    let format = match options.get("format") {
        Some(Value::Null) => JSON_FORMAT.to_owned(),
        Some(Value::String(format)) => format.to_ascii_lowercase(),
        _ => {
            diagnostics.push("Charon LLBC options.format is missing or malformed".to_owned());
            String::new()
        }
    };
    if !unstructured && !not_serialized && format == JSON_FORMAT {
        SERIALIZATION_LLBC_JSON.to_owned()
    } else {
        format!("ullbc={unstructured},no-serialize={not_serialized},format={format}")
    }
}

#[derive(Deserialize)]
struct TranslationJson {
    #[serde(rename = "crate")]
    crate_name: String,
    charon_version: String,
    functions: Vec<TranslationJsonFunction>,
}

// The flags mirror Aeneas `translation.json` keys; they are independent declaration facts.
#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize)]
struct TranslationJsonFunction {
    def_id: u64,
    lean_name: String,
    lean_file: String,
    rust_name: String,
    is_local: bool,
    source: TranslationJsonSource,
    is_opaque: bool,
    can_fail: bool,
    can_diverge: bool,
    is_rec: bool,
}

#[derive(Deserialize)]
struct TranslationJsonSource {
    file: String,
    begin_line: u64,
    end_line: u64,
}

/// A generated Lean file measured by the imperative shell.
pub struct MeasuredLeanFile<'a> {
    /// Locator named by `translation.json` `lean_file`.
    pub locator: &'a str,
    /// Exact measured bytes.
    pub contents: &'a [u8],
}

/// Normalize the produced LLBC and Aeneas `translation.json` into the typed manifest contract.
///
/// Local LLBC function declarations become the current-run join targets; local `translation.json`
/// functions become manifest links. Both sides express their Rust span as the byte envelope of the
/// covered source lines in the LLBC-embedded source file, so they compare equal exactly when the
/// manifest names the same lines Charon extracted.
///
/// # Errors
///
/// Returns diagnostics when either artifact is malformed or the two artifacts name different
/// crates or Charon versions.
// r[impl octet.aeneas_lean_proof_rail.translation_manifest]
pub fn observe_aeneas_translation(
    llbc_text: &str,
    translation_text: &str,
    generated_files: &[MeasuredLeanFile<'_>],
) -> Result<(AeneasTranslationManifest, Vec<AeneasLlbcDeclaration>), Vec<String>> {
    let llbc = serde_json::from_str::<Value>(llbc_text).map_err(|error| vec![format!("parse Charon LLBC: {error}")])?;
    let translation = serde_json::from_str::<TranslationJson>(translation_text)
        .map_err(|error| vec![format!("parse Aeneas translation.json: {error}")])?;
    let mut diagnostics = Vec::new();
    let translated = &llbc["translated"];
    if translated.get("crate_name").and_then(Value::as_str) != Some(translation.crate_name.as_str()) {
        diagnostics.push("translation.json crate does not match the LLBC crate".to_owned());
    }
    if llbc.get("charon_version").and_then(Value::as_str) != Some(translation.charon_version.as_str()) {
        diagnostics.push("translation.json charon_version does not match the LLBC".to_owned());
    }
    let files = llbc_source_files(translated, &mut diagnostics);
    let llbc_declarations = llbc_function_declarations(translated, &files, &mut diagnostics);
    let declarations = translation
        .functions
        .into_iter()
        .filter(|function| function.is_local)
        .map(|function| {
            let rust_span =
                line_envelope(&files, &function.source.file, function.source.begin_line, function.source.end_line)
                    .unwrap_or_else(|| {
                        diagnostics.push(format!(
                            "translation.json function `{}` names lines absent from the LLBC source `{}`",
                            function.rust_name, function.source.file
                        ));
                        empty_span(&function.source.file)
                    });
            AeneasTranslationDeclaration {
                def_id: function.def_id.to_string(),
                rust_name: function.rust_name,
                rust_span,
                lean_name: function.lean_name,
                lean_file: function.lean_file,
                is_opaque: function.is_opaque,
                can_fail: function.can_fail,
                can_diverge: function.can_diverge,
                is_recursive: function.is_rec,
                loop_parent_def_id: None,
                durable_identity: None,
            }
        })
        .collect();
    let mut generated = generated_files
        .iter()
        .map(|file| AeneasGeneratedLeanFile {
            locator: file.locator.to_owned(),
            blake3: blake3::hash(file.contents).to_hex().to_string(),
        })
        .collect::<Vec<_>>();
    generated.sort();
    if diagnostics.is_empty() {
        Ok((
            AeneasTranslationManifest {
                schema_version: AENEAS_TRANSLATION_MANIFEST_SCHEMA_VERSION.to_owned(),
                declarations,
                generated_files: generated,
            },
            llbc_declarations,
        ))
    } else {
        diagnostics.sort();
        diagnostics.dedup();
        Err(diagnostics)
    }
}

/// Observe and validate `translation.json` against the produced LLBC and generated Lean files.
#[must_use]
pub fn validate_observed_aeneas_translation(
    llbc_text: &str,
    translation_text: &str,
    generated_files: &[MeasuredLeanFile<'_>],
) -> Vec<String> {
    match observe_aeneas_translation(llbc_text, translation_text, generated_files) {
        Ok((manifest, llbc_declarations)) => validate_aeneas_translation_manifest(&manifest, &llbc_declarations),
        Err(diagnostics) => diagnostics,
    }
}

struct LlbcSourceFile<'a> {
    id: u64,
    path: &'a str,
    contents: &'a str,
}

fn llbc_source_files<'a>(translated: &'a Value, diagnostics: &mut Vec<String>) -> Vec<LlbcSourceFile<'a>> {
    let Some(files) = translated.get("files").and_then(Value::as_array) else {
        diagnostics.push("Charon LLBC translated.files is missing".to_owned());
        return Vec::new();
    };
    files
        .iter()
        .filter_map(|file| {
            Some(LlbcSourceFile {
                id: file.get("id")?.as_u64()?,
                path: file.pointer("/name/Local")?.as_str()?,
                contents: file.get("contents")?.as_str()?,
            })
        })
        .collect()
}

fn llbc_function_declarations(
    translated: &Value,
    files: &[LlbcSourceFile<'_>],
    diagnostics: &mut Vec<String>,
) -> Vec<AeneasLlbcDeclaration> {
    let Some(functions) = translated.get("fun_decls").and_then(Value::as_array) else {
        diagnostics.push("Charon LLBC translated.fun_decls is missing".to_owned());
        return Vec::new();
    };
    functions
        .iter()
        .filter(|function| function.pointer("/item_meta/is_local").and_then(Value::as_bool) == Some(true))
        .filter_map(|function| {
            let Some(def_id) = function.get("def_id").and_then(Value::as_u64) else {
                diagnostics.push("Charon LLBC local function declaration is missing def_id".to_owned());
                return None;
            };
            let span = function.pointer("/item_meta/span/data");
            let file = span
                .and_then(|span| span.get("file_id"))
                .and_then(Value::as_u64)
                .and_then(|file_id| files.iter().find(|file| file.id == file_id));
            let lines = span
                .and_then(|span| Some((span.pointer("/beg/line")?.as_u64()?, span.pointer("/end/line")?.as_u64()?)));
            let rust_span = file
                .zip(lines)
                .and_then(|(file, (begin, end))| line_envelope(files, file.path, begin, end))
                .unwrap_or_else(|| {
                    diagnostics.push(format!("Charon LLBC function {def_id} has no local source span"));
                    empty_span("")
                });
            Some(AeneasLlbcDeclaration {
                def_id: def_id.to_string(),
                rust_name: rendered_name(function.pointer("/item_meta/name")),
                rust_span,
            })
        })
        .collect()
}

fn rendered_name(name: Option<&Value>) -> String {
    name.and_then(Value::as_array)
        .map(|elements| {
            elements
                .iter()
                .map(|element| {
                    element
                        .pointer("/Ident/0")
                        .and_then(Value::as_str)
                        .map_or_else(|| element.to_string(), str::to_owned)
                })
                .collect::<Vec<_>>()
                .join("::")
        })
        .unwrap_or_default()
}

fn line_envelope(files: &[LlbcSourceFile<'_>], path: &str, begin_line: u64, end_line: u64) -> Option<AeneasSourceSpan> {
    let file = files.iter().find(|file| file.path == path)?;
    let (start_byte, end_byte) = crate::aeneas_consumer::selected_line_range(file.contents, begin_line, end_line)?;
    Some(AeneasSourceSpan {
        locator: path.to_owned(),
        start_byte,
        end_byte,
    })
}

fn empty_span(locator: &str) -> AeneasSourceSpan {
    AeneasSourceSpan {
        locator: locator.to_owned(),
        start_byte: 0,
        end_byte: 0,
    }
}

/// A trust-relevant Lean declaration form found in an Octet-bound Lean source.
#[derive(Clone, Debug, Eq, PartialEq)]
enum LeanTrustForm {
    /// `axiom <name>`.
    Axiom(String),
    /// `opaque <name>`.
    Opaque(String),
    /// `sorry`, `admit`, or `sorryAx`.
    Admitted(String),
    /// `native_decide`, `ofReduceBool`, `implemented_by`, or `extern`, which trust compiled code.
    NativeTrust(String),
}

/// Observe the theorem trust closure from the maintained theorem and generated Lean sources.
///
/// Both sources are scanned as Lean tokens, skipping comments, strings, and character literals.
/// `axiom` declarations are declared only when named by the assumption policy's axioms or
/// external models; `opaque` definitions only when named by its external models. Admitted
/// declarations (`sorry`, `admit`, `sorryAx`) are recorded for rejection, and native-code trust
/// (`native_decide`, `ofReduceBool`, `implemented_by`, `extern`) is always undeclared because the
/// policy has no way to declare it.
// r[impl octet.aeneas_lean_proof_rail.theorem_adequacy]
#[must_use]
pub fn observe_aeneas_trust_closure(
    theorem_source: &str,
    generated_lean: &str,
    policy: &AeneasAssumptionPolicy,
) -> AeneasTrustClosure {
    let mut opaque_definitions = BTreeSet::new();
    let mut admitted = BTreeSet::new();
    let mut undeclared = BTreeSet::new();
    for (origin, source) in [(THEOREM_ORIGIN, theorem_source), (GENERATED_ORIGIN, generated_lean)] {
        for form in lean_trust_forms(source) {
            match form {
                LeanTrustForm::Axiom(name)
                    if policy.axioms.contains(&name) || policy.external_models.contains(&name) => {}
                LeanTrustForm::Axiom(name) => {
                    undeclared.insert(format!("{origin}: axiom {name}"));
                }
                LeanTrustForm::Opaque(name) if policy.external_models.contains(&name) => {
                    opaque_definitions.insert(name);
                }
                LeanTrustForm::Opaque(name) => {
                    undeclared.insert(format!("{origin}: opaque {name}"));
                }
                LeanTrustForm::Admitted(token) => {
                    admitted.insert(format!("{origin}: {token}"));
                }
                LeanTrustForm::NativeTrust(token) => {
                    undeclared.insert(format!("{origin}: {token}"));
                }
            }
        }
    }
    AeneasTrustClosure {
        complete: true,
        declared_external_models: policy.external_models.clone(),
        declared_axioms: policy.axioms.clone(),
        declared_opaque_definitions: opaque_definitions.into_iter().collect(),
        declared_admitted_declarations: admitted.into_iter().collect(),
        undeclared_inputs: undeclared.into_iter().collect(),
    }
}

fn lean_trust_forms(source: &str) -> Vec<LeanTrustForm> {
    let tokens = lean_identifier_tokens(source);
    let mut forms = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let next = || tokens.get(index + 1).cloned().unwrap_or_default();
        match token.as_str() {
            "axiom" => forms.push(LeanTrustForm::Axiom(next())),
            "opaque" => forms.push(LeanTrustForm::Opaque(next())),
            "sorry" | "admit" | "sorryAx" => forms.push(LeanTrustForm::Admitted(token.clone())),
            "native_decide" | "ofReduceBool" | "Lean.ofReduceBool" | "extern" => {
                forms.push(LeanTrustForm::NativeTrust(token.clone()));
            }
            "implemented_by" => forms.push(LeanTrustForm::NativeTrust(format!("implemented_by {}", next()))),
            _ => {}
        }
    }
    forms
}

/// Split Lean source into identifier tokens, skipping comments, strings, and character literals.
fn lean_identifier_tokens(source: &str) -> Vec<String> {
    let characters = source.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        let next = characters.get(index + 1).copied();
        if character == '-' && next == Some('-') {
            while index < characters.len() && characters[index] != '\n' {
                index += 1;
            }
        } else if character == '/' && next == Some('-') {
            index = skip_block_comment(&characters, index);
        } else if character == '"' {
            index = skip_string(&characters, index + 1);
        } else if character == 'r' && raw_string_start(&characters, index) {
            index = skip_raw_string(&characters, index + 1);
        } else if character == '\'' {
            index = skip_character_literal(&characters, index);
        } else if character == '«' {
            let start = index + 1;
            index = start;
            while index < characters.len() && characters[index] != '»' {
                index += 1;
            }
            tokens.push(characters[start..index].iter().collect());
            index += 1;
        } else if identifier_start(character) {
            let start = index;
            index += 1;
            while index < characters.len()
                && (identifier_continue(characters[index])
                    || (characters[index] == '.' && characters.get(index + 1).copied().is_some_and(identifier_start)))
            {
                index += 1;
            }
            tokens.push(characters[start..index].iter().collect());
        } else {
            index += 1;
        }
    }
    tokens
}

fn skip_block_comment(characters: &[char], start: usize) -> usize {
    let mut depth = 0_usize;
    let mut index = start;
    while index < characters.len() {
        let pair = (characters[index], characters.get(index + 1).copied());
        if pair == ('/', Some('-')) {
            depth += 1;
            index += 2;
        } else if pair == ('-', Some('/')) {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return index;
            }
        } else {
            index += 1;
        }
    }
    index
}

fn skip_string(characters: &[char], mut index: usize) -> usize {
    while index < characters.len() {
        match characters[index] {
            '\\' => index += 2,
            '"' => return index + 1,
            _ => index += 1,
        }
    }
    index
}

fn raw_string_start(characters: &[char], index: usize) -> bool {
    if index > 0 && identifier_continue(characters[index - 1]) {
        return false;
    }
    let mut cursor = index + 1;
    while characters.get(cursor) == Some(&'#') {
        cursor += 1;
    }
    characters.get(cursor) == Some(&'"')
}

fn skip_raw_string(characters: &[char], mut index: usize) -> usize {
    let mut hashes = 0;
    while characters.get(index) == Some(&'#') {
        hashes += 1;
        index += 1;
    }
    index += 1;
    while index < characters.len() {
        if characters[index] == '"' && (1..=hashes).all(|offset| characters.get(index + offset) == Some(&'#')) {
            return index + 1 + hashes;
        }
        index += 1;
    }
    index
}

fn skip_character_literal(characters: &[char], index: usize) -> usize {
    // A quote directly after an identifier character is part of that identifier (`h'`), and the
    // identifier scanner already consumed it; any other quote opens a character literal.
    let body = index + 1;
    let close = if characters.get(body) == Some(&'\\') {
        (body + 1..characters.len()).find(|&cursor| characters[cursor] == '\'')
    } else {
        Some(body + 1).filter(|&cursor| characters.get(cursor) == Some(&'\''))
    };
    close.map_or(index + 1, |cursor| cursor + 1)
}

fn identifier_start(character: char) -> bool {
    character.is_alphabetic() || character == '_'
}

fn identifier_continue(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '\'' | '!' | '?')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal_proof::validate_charon_extraction_contract;

    const FIXTURE_ROOT: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas/"
    );
    const PROFILE_JSON: &str = include_str!("../../../standards/proof-rails/aeneas-lean/generated/profile.json");

    fn fixture(name: &str) -> String {
        std::fs::read_to_string(format!("{FIXTURE_ROOT}{name}")).expect("frozen Aeneas fixture artifact")
    }

    fn reviewed_policy() -> CharonExtractionPolicy {
        let profile = serde_json::from_str::<Value>(PROFILE_JSON).expect("generated provider profile");
        serde_json::from_value(profile["extraction_policy"].clone()).expect("reviewed extraction policy")
    }

    fn empty_policy() -> AeneasAssumptionPolicy {
        AeneasAssumptionPolicy {
            external_models: Vec::new(),
            axioms: Vec::new(),
            requires_complete_declaration: true,
        }
    }

    // r[verify octet.aeneas_lean_proof_rail.charon_contract]
    #[test]
    fn aeneas_observed_charon_policy_matches_reviewed_contract_for_real_llbc() {
        let observed = observe_charon_extraction_policy(&fixture("llbc.llbc")).expect("observed policy");

        assert!(observed.include_rules.is_empty(), "the executed Charon argv applies no include rules");
        assert_eq!(validate_charon_extraction_contract(&observed, &reviewed_policy()), Vec::<String>::new());
    }

    /// A named LLBC mutation and the extraction-policy field it must drift.
    type LlbcMutation = (&'static str, fn(&mut Value));

    // r[verify octet.aeneas_lean_proof_rail.charon_contract]
    #[test]
    fn aeneas_observed_charon_policy_rejects_weakened_or_missing_options() {
        let original = serde_json::from_str::<Value>(&fixture("llbc.llbc")).expect("LLBC JSON");
        let mutations: [LlbcMutation; 9] = [
            ("has_errors", |llbc| llbc["has_errors"] = Value::Bool(true)),
            ("skip_borrowck", |llbc| llbc["translated"]["options"]["skip_borrowck"] = Value::Bool(true)),
            ("pointer_width_bits", |llbc| {
                llbc["translated"]["target_information"][0]["value"]["target_pointer_size"] = Value::from(4_u64);
            }),
            ("endianness", |llbc| {
                llbc["translated"]["target_information"][0]["value"]["is_little_endian"] = Value::Bool(false);
            }),
            ("mir_start_stage", |llbc| llbc["translated"]["options"]["mir"] = Value::from("Optimized")),
            ("drop_policy", |llbc| llbc["translated"]["options"]["precise_drops"] = Value::Bool(true)),
            ("start_from", |llbc| {
                llbc["translated"]["options"]["start_from"] = serde_json::json!(["real_fixture::other"]);
            }),
            ("transformation_settings", |llbc| {
                llbc["translated"]["fun_decls"][0]["item_meta"].as_object_mut().expect("item meta").remove("span");
            }),
            ("serialization_format", |llbc| llbc["translated"]["options"]["ullbc"] = Value::Bool(true)),
        ];
        for (field, mutate) in mutations {
            let mut llbc = original.clone();
            mutate(&mut llbc);
            let observed = observe_charon_extraction_policy(&llbc.to_string()).expect("observed mutated policy");
            let diagnostics = validate_charon_extraction_contract(&observed, &reviewed_policy());
            assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(field)), "{field}: {diagnostics:?}");
        }

        let mut missing = original;
        missing["translated"]["options"].as_object_mut().expect("options").remove("skip_borrowck");
        assert!(observe_charon_extraction_policy(&missing.to_string()).is_err());
        assert!(observe_charon_extraction_policy("not json").is_err());
    }

    // r[verify octet.aeneas_lean_proof_rail.translation_manifest]
    #[test]
    fn aeneas_observed_translation_links_real_llbc_and_generated_lean() {
        let generated = fixture("Llbc.lean");
        let files = [MeasuredLeanFile {
            locator: "Llbc.lean",
            contents: generated.as_bytes(),
        }];
        let diagnostics =
            validate_observed_aeneas_translation(&fixture("llbc.llbc"), &fixture("translation.json"), &files);

        assert_eq!(diagnostics, Vec::<String>::new());
    }

    // r[verify octet.aeneas_lean_proof_rail.translation_manifest]
    #[test]
    fn aeneas_observed_translation_rejects_stale_spans_names_files_and_crates() {
        let generated = fixture("Llbc.lean");
        let files = [MeasuredLeanFile {
            locator: "Llbc.lean",
            contents: generated.as_bytes(),
        }];
        let llbc = fixture("llbc.llbc");
        let original = serde_json::from_str::<Value>(&fixture("translation.json")).expect("translation JSON");
        let mutations: [fn(&mut Value); 5] = [
            |translation| translation["functions"][0]["source"]["end_line"] = Value::from(2_u64),
            |translation| translation["functions"][0]["rust_name"] = Value::from("real_fixture::other"),
            |translation| translation["functions"][0]["def_id"] = Value::from(7_u64),
            |translation| translation["functions"][0]["lean_file"] = Value::from("Other.lean"),
            |translation| translation["crate"] = Value::from("other_crate"),
        ];
        for mutate in mutations {
            let mut translation = original.clone();
            mutate(&mut translation);
            let diagnostics = validate_observed_aeneas_translation(&llbc, &translation.to_string(), &files);
            assert!(!diagnostics.is_empty(), "mutated translation unexpectedly validated: {translation}");
        }
        assert!(!validate_observed_aeneas_translation(&llbc, &original.to_string(), &[]).is_empty());
        assert!(!validate_observed_aeneas_translation(&llbc, "{}", &files).is_empty());
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_trust_closure_accepts_clean_sources_and_ignores_comments_and_strings() {
        let theorem = "import Llbc\n/-- no axiom here; sorry is prose -/\ntheorem t : f 1 = .ok 2 := by\n  -- native_decide would be wrong\n  rfl\ndef s := \"axiom cheat\"\ndef c := 'a'\ntheorem h' : True := trivial\n";
        let closure = observe_aeneas_trust_closure(theorem, &fixture("Llbc.lean"), &empty_policy());

        assert!(closure.complete);
        assert_eq!(closure.undeclared_inputs, Vec::<String>::new());
        assert_eq!(closure.declared_admitted_declarations, Vec::<String>::new());
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_trust_closure_reports_undeclared_axioms_opaque_and_native_trust() {
        let theorem = "axiom cheat : 1 = 2\ntheorem t : 1 = 2 := cheat\nopaque hidden : Nat\ntheorem n : 2 = 2 := by native_decide\n@[implemented_by fast] def slow := 1\n";
        let generated = "/- generated -/\naxiom core.model : Nat\ntheorem g : True := sorry\n";
        let closure = observe_aeneas_trust_closure(theorem, generated, &empty_policy());

        assert_eq!(closure.undeclared_inputs, vec![
            "generated: axiom core.model".to_owned(),
            "theorem: axiom cheat".to_owned(),
            "theorem: implemented_by fast".to_owned(),
            "theorem: native_decide".to_owned(),
            "theorem: opaque hidden".to_owned(),
        ]);
        assert_eq!(closure.declared_admitted_declarations, vec!["generated: sorry".to_owned()]);

        let declared = AeneasAssumptionPolicy {
            external_models: vec!["core.model".to_owned(), "hidden".to_owned()],
            axioms: vec!["cheat".to_owned()],
            requires_complete_declaration: true,
        };
        let closure = observe_aeneas_trust_closure(theorem, "axiom core.model : Nat\n", &declared);
        assert_eq!(closure.undeclared_inputs, vec![
            "theorem: implemented_by fast".to_owned(),
            "theorem: native_decide".to_owned()
        ]);
        assert_eq!(closure.declared_opaque_definitions, vec!["hidden".to_owned()]);
    }
}
