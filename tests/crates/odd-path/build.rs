fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-env=GREETING=from-build-script");
    println!("cargo::rustc-check-cfg=cfg(has_gen)");
    println!("cargo::rustc-cfg=has_gen");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(format!("{out_dir}/gen.rs"), "pub fn generated() -> &'static str { \"generated-text\" }\n").unwrap();
}
