fn main() -> Result<(), String> {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = crate_root
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or("architecture IR crate has no repository root")?;
    let output_path = repo_root.join(octet_architecture_ir::COMPILER_ARCHITECTURE_CONTRACT_PATH);
    let output = octet_architecture_ir::render_nickel_contract();
    std::fs::write(&output_path, output).map_err(|error| format!("write {}: {error}", output_path.display()))?;
    println!("{}", output_path.display());
    Ok(())
}
