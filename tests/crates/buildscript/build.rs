fn main() {
    println!("cargo::rerun-if-changed=data.txt");
    println!("cargo::rerun-if-env-changed=LUST_FIXTURE_FLAG");
    println!("cargo::rustc-env=GREETING=from-build-script");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let data = std::fs::read_to_string("data.txt").unwrap_or_default();
    std::fs::write(format!("{out_dir}/generated.txt"), data).unwrap();
}
