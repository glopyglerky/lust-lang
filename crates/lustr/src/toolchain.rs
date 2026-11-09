use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub(crate) fn program(root: &Path, name: &str) -> PathBuf {
    root.join("bin").join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}

pub(crate) fn environment(root: &Path) -> Vec<(OsString, OsString)> {
    [("CARGO", "cargo"), ("LUST_CARGO", "cargo"), ("RUSTC", "rustc"), ("RUSTDOC", "rustdoc")]
        .into_iter()
        .map(|(variable, executable)| (variable.into(), program(root, executable).into_os_string()))
        .collect()
}

pub(crate) fn validate(root: &Path) -> Result<(), String> {
    for name in ["cargo", "rustc", "rustdoc"] {
        let path = program(root, name);
        if !path.is_file() {
            return Err(format!("BlueFe toolchain is missing {}", path.display()));
        }
    }
    Ok(())
}
