fn main() {
    println!("cargo::rerun-if-env-changed=LUST_SCEN_DEP");
    let v = std::env::var("LUST_SCEN_DEP").unwrap_or_else(|_| "unset".to_string());
    println!("cargo::rustc-env=DEP_VAL=[{v}]");
}
