#![feature(register_tool)]
#![register_tool(tigerstyle)]

use std::fmt::Write as _;

const HASH_INPUT_SEPARATOR: u8 = 0;
const FILE_INPUT_SEPARATOR: u8 = 0xff;
const EMBEDDED_DIRECTORIES: &[&str] = &[
    "src",
    "cargo-octet",
    "crates/octet-architecture-ir",
    "crates/octet-lint-metadata",
    "crates/octet-standards",
    "standards/clippy-restrictions/generated",
    "standards/compiler-architecture-ir/contracts",
    "standards/effect-closure/generated",
    "standards/nominal-domains/generated",
    "standards/proof-imports/generated",
    "standards/strict-profiles/generated",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = cargo_manifest_dir()?;
    let repo_root = manifest_dir
        .parent()
        .ok_or_else(|| format!("cargo-octet manifest has no parent: {}", manifest_dir.display()))?;

    let mut files = Vec::new();
    let mut add_file = |path: &std::path::Path, relative: &std::path::Path| files.push(collect_file(path, relative));
    add_file(&repo_root.join("Cargo.toml"), std::path::Path::new("Cargo.toml"));
    add_file(&repo_root.join("Cargo.lock"), std::path::Path::new("Cargo.lock"));
    add_file(&repo_root.join("rust-toolchain.toml"), std::path::Path::new("rust-toolchain.toml"));
    for relative in EMBEDDED_DIRECTORIES {
        collect_dir(&repo_root.join(relative), std::path::Path::new(relative), &mut add_file)?;
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let mut hasher = blake3::Hasher::new();
    for (relative, absolute) in &files {
        let bytes = std::fs::read(absolute).map_err(|error| format!("read {}: {error}", absolute.display()))?;
        hasher.update(relative.as_bytes());
        hasher.update(&[HASH_INPUT_SEPARATOR]);
        hasher.update(&bytes);
        hasher.update(&[FILE_INPUT_SEPARATOR]);
    }
    let hash = hasher.finalize().to_hex().to_string();

    let out_path = cargo_out_dir()?.join("embedded_source.rs");
    let mut generated = String::new();
    writeln!(&mut generated, "pub const EMBEDDED_SOURCE_HASH: &str = {hash:?};")?;
    generated.push_str("pub const EMBEDDED_SOURCE_FILES: &[(&str, &[u8])] = &[\n");
    for (relative, absolute) in &files {
        let absolute_display = absolute.display().to_string();
        writeln!(&mut generated, "    ({relative:?}, include_bytes!({absolute_display:?}) as &[u8]),")?;
    }
    generated.push_str("];\n");
    std::fs::write(&out_path, generated).map_err(|error| format!("write {}: {error}", out_path.display()))?;

    println!("cargo:rerun-if-changed={}", repo_root.join("Cargo.toml").display());
    println!("cargo:rerun-if-changed={}", repo_root.join("Cargo.lock").display());
    println!("cargo:rerun-if-changed={}", repo_root.join("rust-toolchain.toml").display());
    for relative in EMBEDDED_DIRECTORIES {
        println!("cargo:rerun-if-changed={}", repo_root.join(relative).display());
    }
    Ok(())
}

#[expect(
    tigerstyle::ambient_env,
    reason = "Cargo build script reads CARGO_MANIFEST_DIR only at execution; owner: octet maintainers"
)]
fn cargo_manifest_dir() -> Result<std::path::PathBuf, std::io::Error> {
    std::env::var_os("CARGO_MANIFEST_DIR").map(std::path::PathBuf::from).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Cargo build script is missing CARGO_MANIFEST_DIR")
    })
}

#[expect(
    tigerstyle::ambient_env,
    reason = "Cargo build script reads OUT_DIR only at execution; owner: octet maintainers"
)]
fn cargo_out_dir() -> Result<std::path::PathBuf, std::io::Error> {
    std::env::var_os("OUT_DIR")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Cargo build script is missing OUT_DIR"))
}

fn collect_dir(
    root: &std::path::Path,
    relative_root: &std::path::Path,
    add_file: &mut impl FnMut(&std::path::Path, &std::path::Path),
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pending = vec![(root.to_path_buf(), relative_root.to_path_buf())];
    while let Some((directory, relative_directory)) = pending.pop() {
        let entries =
            std::fs::read_dir(&directory).map_err(|error| format!("read {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("dir entry under {}: {error}", directory.display()))?;
            let entry_path = entry.path();
            let relative = relative_directory.join(entry.file_name());
            if entry
                .file_type()
                .map_err(|error| format!("file type for {}: {error}", entry_path.display()))?
                .is_dir()
            {
                pending.push((entry_path, relative));
            } else {
                add_file(&entry_path, &relative);
            }
        }
    }
    Ok(())
}

fn collect_file(path: &std::path::Path, relative: &std::path::Path) -> (String, std::path::PathBuf) {
    (relative.to_string_lossy().replace('\\', "/"), path.to_path_buf())
}
