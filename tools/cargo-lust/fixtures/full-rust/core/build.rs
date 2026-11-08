use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=FIXTURE_BUILD_VALUE=7");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"));
    fs::write(out.join("generated.rs"), "pub const GENERATED: u32 = 7;\n").unwrap();
}
