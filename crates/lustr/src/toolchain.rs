use std::ffi::{OsStr, OsString};
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
            return Err(format!("selected toolchain is missing {}", path.display()));
        }
    }
    Ok(())
}

pub(crate) fn packaged_root() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    let bin = executable.parent()?;
    if bin.file_name() != Some(OsStr::new("bin")) {
        return None;
    }
    let root = bin.parent()?;
    if !root.join("lustr-toolchain.json").is_file() {
        return None;
    }
    if ["LUST_CARGO", "RUSTC", "RUSTDOC"].iter().any(|name| std::env::var_os(name).is_some()) {
        return None;
    }
    if let Some(cargo) = std::env::var_os("CARGO") {
        let bundled = program(root, "cargo");
        let selected = Path::new(&cargo);
        if selected != bundled && selected.canonicalize().ok().zip(bundled.canonicalize().ok()).is_none_or(|(left, right)| left != right) {
            return None;
        }
    }
    Some(root.to_path_buf())
}
