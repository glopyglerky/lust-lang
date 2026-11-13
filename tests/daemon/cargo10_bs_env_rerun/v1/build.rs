fn main() {
    println!("cargo::rerun-if-env-changed=LUST_SCEN_BS");
    println!("cargo::rustc-check-cfg=cfg(bs_a)");
    let v = std::env::var("LUST_SCEN_BS").unwrap_or_else(|_| "unset".to_string());
    println!("cargo::rustc-env=BS_VAL=[{v}]");
    if v == "a" {
        println!("cargo::rustc-cfg=bs_a");
    }
}
