// feature: ext.serde_json
// oracle: lust-only
// form: serde_json::from_str into a plain type, as in ts-rust codegen tools
fn main() {
    let s: String = serde_json::from_str("\"a\"").map_err(|e| e.to_string()).unwrap_or_default();
    println!("{s}");
}
