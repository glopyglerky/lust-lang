// expect: L0101
// rustc error here: failed to resolve `serde_json` (no such crate in
// a single file). With serde_json linked, lust still has no adapter
// for it: serde_json is a named gap (feature ext.serde_json).
fn main() {
    let s: String = serde_json::from_str("\"a\"").unwrap_or_default();
    println!("{s}");
}
