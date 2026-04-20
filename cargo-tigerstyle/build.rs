use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().expect("cargo-tigerstyle repo root").to_path_buf();

    let mut files = Vec::new();
    collect_file(&repo_root.join("Cargo.toml"), Path::new("Cargo.toml"), &mut files);
    collect_file(&repo_root.join("Cargo.lock"), Path::new("Cargo.lock"), &mut files);
    collect_file(&repo_root.join("rust-toolchain.toml"), Path::new("rust-toolchain.toml"), &mut files);
    collect_dir(&repo_root.join("src"), Path::new("src"), &mut files);
    collect_dir(&manifest_dir, Path::new("cargo-tigerstyle"), &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let mut hasher = blake3::Hasher::new();
    for (relative, absolute) in &files {
        let bytes = fs::read(absolute).unwrap_or_else(|error| panic!("read {}: {error}", absolute.display()));
        hasher.update(relative.as_bytes());
        hasher.update(&[0]);
        hasher.update(&bytes);
        hasher.update(&[0xff]);
    }
    let hash = hasher.finalize().to_hex().to_string();

    let out_path = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("embedded_source.rs");
    let mut generated = String::new();
    generated.push_str(&format!("pub const EMBEDDED_SOURCE_HASH: &str = {:?};\n", hash));
    generated.push_str("pub const EMBEDDED_SOURCE_FILES: &[(&str, &[u8])] = &[\n");
    for (relative, absolute) in &files {
        generated.push_str(&format!(
            "    ({:?}, include_bytes!({:?}) as &[u8]),\n",
            relative,
            absolute.display().to_string()
        ));
    }
    generated.push_str("];\n");
    fs::write(&out_path, generated).unwrap_or_else(|error| panic!("write {}: {error}", out_path.display()));

    println!("cargo:rerun-if-changed={}", repo_root.join("Cargo.toml").display());
    println!("cargo:rerun-if-changed={}", repo_root.join("Cargo.lock").display());
    println!("cargo:rerun-if-changed={}", repo_root.join("rust-toolchain.toml").display());
    println!("cargo:rerun-if-changed={}", repo_root.join("src").display());
    println!("cargo:rerun-if-changed={}", manifest_dir.display());
}

fn collect_dir(root: &Path, relative_root: &Path, files: &mut Vec<(String, PathBuf)>) {
    for entry in fs::read_dir(root).unwrap_or_else(|error| panic!("read {}: {error}", root.display())) {
        let entry = entry.unwrap_or_else(|error| panic!("dir entry under {}: {error}", root.display()));
        let entry_path = entry.path();
        let relative = relative_root.join(entry.file_name());
        if entry
            .file_type()
            .unwrap_or_else(|error| panic!("file type for {}: {error}", entry_path.display()))
            .is_dir()
        {
            collect_dir(&entry_path, &relative, files);
        } else {
            collect_file(&entry_path, &relative, files);
        }
    }
}

fn collect_file(path: &Path, relative: &Path, files: &mut Vec<(String, PathBuf)>) {
    files.push((relative.to_string_lossy().replace('\\', "/"), path.to_path_buf()));
}
