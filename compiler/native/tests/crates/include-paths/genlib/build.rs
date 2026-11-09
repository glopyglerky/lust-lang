fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(format!("{out_dir}/dep.txt"), "dep-out-data").unwrap();
}
