//! Regenerates deterministic checked-in JSON fixtures for `standards/structural-linkage/`.

use std::fs;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root").to_path_buf()
}

fn canonical_fn_hash(src: &str, fn_name: &str) -> String {
    let ast = syn::parse_file(src).expect("valid rust source");
    for item in &ast.items {
        if let syn::Item::Fn(func) = item
            && func.sig.ident == fn_name
        {
            return super::source::normalized_item_hash(item);
        }
    }
    panic!("missing fn `{fn_name}`");
}

fn cairn_binding_hash(binding: &serde_json::Value) -> String {
    super::domain_hash_json(
        "cairn.trace-index.rust-item-binding.v1",
        &(
            binding["link_blake3"].as_str().expect("link"),
            binding["requirement_id"].as_str().expect("req"),
            binding["requirement_row_blake3"].as_str().expect("row"),
            binding["source_path"].as_str().expect("src"),
            binding["source_file_blake3"].as_str().expect("src_hash"),
            &binding["marker_span"],
            &binding["item"],
            "cairn.saved-rust-bytes.v1",
            "syn-2.0.117",
            "cairn.rust-item-binding.v1",
            &binding["configuration"],
        ),
    )
    .expect("compute cairn binding hash")
}

pub(super) fn build_cairn_v2_report(source_path: &str, source_text: &str, include_history: bool) -> Vec<u8> {
    let file_hash = super::blake3_hex(source_text.as_bytes());
    let req_id = "octet.structural_linkage.pilot.checked_add";
    let row_hash = super::blake3_hex(b"row-checked-add");
    let marker_start = source_text.find("r[impl ").expect("marker present") as u64;
    let marker_span = serde_json::json!({"start_byte": marker_start, "end_byte": marker_start.saturating_add(44), "start_line": 1, "end_line": 1});
    let link_blake3 = super::blake3_hex(format!("{req_id}:{file_hash}").as_bytes());
    let config = serde_json::json!({"source_path": source_path, "file_blake3": file_hash, "schema_version": "cairn.rust-item-binding.v1", "target": "x86_64-unknown-linux-gnu", "features": [], "declared_features": [], "cfg": []});
    let mut binding = serde_json::json!({
        "link_blake3": link_blake3, "requirement_id": req_id, "requirement_row_blake3": row_hash,
        "source_path": source_path, "source_file_blake3": file_hash, "marker_span": marker_span,
        "item": {"source_path": source_path, "marker_start_byte": marker_start, "start_byte": source_text.find("pub fn checked_add").expect("fn") as u64, "end_byte": source_text.len() as u64, "item_kind": "fn", "scoped_locator": "pilot/fn:checked_add#0", "normalized_item_blake3": canonical_fn_hash(source_text, "checked_add")},
        "configuration": config, "binding_blake3": "",
    });
    binding["binding_blake3"] = serde_json::Value::String(cairn_binding_hash(&binding));
    let mut history_vec = Vec::new();
    if include_history {
        let mut hist = binding.clone();
        hist["requirement_id"] = serde_json::Value::String("octet.structural_linkage.pilot.old_req".to_owned());
        hist["binding_blake3"] = serde_json::Value::String(cairn_binding_hash(&hist));
        history_vec.push(hist);
    }
    let mut root = serde_json::json!({
        "schema_version": "cairn.content-bound-trace-index.v2", "boundary": "content-bound trace index",
        "profile": {"name": "cairn-pilot"}, "profile_blake3": super::blake3_hex(b"profile"),
        "policy_blake3": super::blake3_hex(b"policy"), "requirement_registry_blake3": super::blake3_hex(b"registry"),
        "requirements": [{"requirement_id": req_id, "row_blake3": row_hash}],
        "sources": [{"path": source_path, "content_blake3": file_hash}],
        "links": [{"requirement_id": req_id, "requirement_row_blake3": row_hash, "role": "impl", "source_path": source_path, "source_file_blake3": file_hash, "marker_span": marker_span, "marker_blake3": super::blake3_hex(b"// r[impl octet.structural_linkage.pilot.checked_add]"), "link_blake3": link_blake3}],
        "non_claims": ["not callee behavior", "not reusable proof freshness"],
        "structural": {"adapter": "cairn.saved-rust-bytes.v1", "parser": "syn-2.0.117", "schema": "cairn.rust-item-binding.v1", "configuration": config, "bindings": [binding], "history": history_vec}
    });
    let index_hash = super::domain_hash_json("cairn.trace-index.structural-index.v2", &root).expect("index hash");
    root["index_blake3"] = serde_json::Value::String(index_hash);
    serde_json::to_vec_pretty(&root).expect("serialize cairn v2 fixture")
}

#[test]
fn checked_structural_fixtures_match_current_intake() {
    let root = repo_root();
    let base = root.join("standards/structural-linkage");
    let valid_dir = base.join("fixtures/valid");

    let target_rel = "standards/structural-linkage/fixtures/target.rs";
    let target_text = fs::read_to_string(root.join(target_rel)).expect("read target.rs");
    let report_bytes = build_cairn_v2_report(target_rel, &target_text, true);
    let profiles_bytes = fs::read(base.join("generated/profiles.json")).expect("read profiles.json");
    let registry = super::parse_profile_registry(&profiles_bytes).expect("parse profiles");
    let profile = super::select_profile(&registry, "cairn-structural-trace-v2").expect("select cairn");
    let (projection, receipt) =
        super::receipt::ingest_report(profile, &report_bytes, &root).expect("ingest cairn fixture");
    assert_eq!(receipt.status, super::STATUS_VALID);
    assert_eq!(fs::read(valid_dir.join("cairn-report.json")).expect("read cairn-report"), report_bytes);
    assert_eq!(
        fs::read(valid_dir.join("projection.json")).expect("read projection"),
        serde_json::to_vec_pretty(&projection).expect("proj")
    );
    assert_eq!(
        fs::read(valid_dir.join("receipt.json")).expect("read receipt"),
        serde_json::to_vec_pretty(&receipt).expect("rcpt")
    );

    let buoy_bytes = super::tests::build_buoy_pilot_report("current", false);
    let buoy_profile = super::select_profile(&registry, "buoy-comparison-pilot-v1").expect("select buoy");
    let (buoy_proj, buoy_rcpt) = super::receipt::ingest_report(buoy_profile, &buoy_bytes, &root).expect("ingest buoy");
    assert_eq!(buoy_rcpt.status, super::STATUS_VALID);
    assert_eq!(fs::read(valid_dir.join("buoy-pilot.json")).expect("read buoy-pilot"), buoy_bytes);
    assert_eq!(
        fs::read(valid_dir.join("buoy-projection.json")).expect("read buoy-projection"),
        serde_json::to_vec_pretty(&buoy_proj).expect("buoy proj")
    );
    assert_eq!(
        fs::read(valid_dir.join("buoy-receipt.json")).expect("read buoy-receipt"),
        serde_json::to_vec_pretty(&buoy_rcpt).expect("buoy rcpt")
    );
}
