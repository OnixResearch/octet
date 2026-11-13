use std::path::Path;

pub(super) fn prepare(
    scratch: &Path,
    runtime: &super::Runtime,
) -> Result<Vec<(std::ffi::OsString, std::ffi::OsString)>, String> {
    std::fs::create_dir(scratch)
        .map_err(|error| format!("create experiment scratch {}: {error}", scratch.display()))?;
    let target = scratch.join("target");
    let home = scratch.join("home");
    let temp = scratch.join("tmp");
    for directory in [&target, &home, &temp] {
        if let Err(error) = std::fs::create_dir(directory) {
            let reason = format!("prepare experiment scratch {}: {error}", directory.display());
            return Err(match cleanup(scratch) {
                Ok(()) => reason,
                Err(cleanup_error) => format!("{reason}; {cleanup_error}"),
            });
        }
    }
    Ok(super::shell::adoption_environment(&home, &temp, &target, runtime))
}

pub(super) fn cleanup(scratch: &Path) -> Result<(), String> {
    std::fs::remove_dir_all(scratch).map_err(|error| format!("clean experiment scratch {}: {error}", scratch.display()))
}
