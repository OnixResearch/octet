use std::path::Path;

fn load_core_config(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|_error| String::new())
}

fn load_wrapped_config(path: &Path) -> std::io::Result<Option<String>> {
    Ok(Some(std::fs::read_to_string(path)?))
}

fn main() {
    let _config = load_core_config(Path::new("config.txt"));
    let _wrapped = load_wrapped_config(Path::new("config.txt"));
}
