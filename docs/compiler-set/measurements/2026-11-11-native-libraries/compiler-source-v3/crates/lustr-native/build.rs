use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

// Run the original build recipe, including its exact TinyCC linkage and runtime
// inputs. This imports source, it does not generate or apply a source patch.
mod upstream {
    include!("../../compiler/native/build.rs");
    pub fn run() { main(); }
}

fn hash_tree(root: &Path, dir: &Path, h: &mut impl Hasher) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).expect("source directory")
        .map(|e| e.expect("source entry").path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() { hash_tree(root, &path, h); }
        else {
            path.strip_prefix(root).expect("source path").hash(h);
            std::fs::read(&path).expect("source bytes").hash(h);
        }
    }
}

fn environment_names(dir: &Path, names: &mut std::collections::BTreeSet<String>) {
    for entry in std::fs::read_dir(dir).expect("native source directory") {
        let path = entry.expect("native source file").path();
        if path.is_dir() { environment_names(&path, names); continue; }
        if path.extension().is_none_or(|e| e != "rs") { continue; }
        let text = std::fs::read_to_string(path).expect("Rust source");
        for tail in text.split("\"LUST_").skip(1) {
            let suffix: String = tail.chars().take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_').collect();
            if !suffix.is_empty() { names.insert(format!("LUST_{suffix}")); }
        }
    }
}

fn main() {
    let own = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let native = own.join("../../compiler/native").canonicalize().expect("upstream source");
    // SAFETY: This build script is single-threaded and changes its own process
    // environment before upstream creates any workers. Restore it immediately.
    unsafe { std::env::set_var("CARGO_MANIFEST_DIR", &native); }
    upstream::run();
    unsafe { std::env::set_var("CARGO_MANIFEST_DIR", &own); }

    // The upstream script's hash excludes this crate. Re-key native caches for
    // both source trees, build recipes, manifests, lockfile and linked TinyCC.
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for base in [&native, &own] {
        hash_tree(base, &base.join("src"), &mut h);
        for name in ["Cargo.toml", "Cargo.lock", "build.rs"] {
            std::fs::read(base.join(name)).expect("build input").hash(&mut h);
        }
    }
    hash_tree(&native, &native.join("library"), &mut h);
    let tcc = std::env::var_os("LUST_TCC_HOME").map(PathBuf::from)
        .unwrap_or_else(|| native.join("third_party/tcc"));
    hash_tree(&tcc, &tcc.join("lib"), &mut h);
    hash_tree(&tcc, &tcc.join("bin"), &mut h);
    for key in ["PROFILE", "OPT_LEVEL", "DEBUG", "TARGET", "HOST", "RUSTC",
                "CARGO_ENCODED_RUSTFLAGS", "CARGO_CFG_TARGET_FEATURE"] {
        key.hash(&mut h);
        std::env::var_os(key).hash(&mut h);
    }
    let mut names = std::collections::BTreeSet::new();
    environment_names(&native.join("src"), &mut names);
    environment_names(&own.join("src"), &mut names);
    let names: Vec<_> = names.into_iter().collect();
    let env_source = format!("const NATIVE_ENV_INPUTS: &[&str] = &{names:?};\n");
    std::fs::write(PathBuf::from(std::env::var_os("OUT_DIR").expect("out dir")).join("native-env-inputs.rs"), env_source).expect("environment input metadata");
    println!("cargo:rustc-env=LUST_BUILD_HASH={:016x}", h.finish());
    println!("cargo:rustc-env=CARGO_MANIFEST_DIR={}", native.display());
    println!("cargo:rerun-if-changed={}", native.join("src").display());
    println!("cargo:rerun-if-changed={}", native.join("library").display());
    println!("cargo:rerun-if-changed={}", native.join("build.rs").display());
    println!("cargo:rerun-if-changed={}", native.join("Cargo.toml").display());
    println!("cargo:rerun-if-changed={}", native.join("Cargo.lock").display());
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.lock");
}
