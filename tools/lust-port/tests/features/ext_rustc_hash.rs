// feature: ext.rustc_hash
// form: FxHashMap type and insert/get, as ts-rust writes it in most files
// cargo-dep: rustc-hash = "=2.1.2"
use rustc_hash::FxHashMap;
fn main() {
    let mut m: FxHashMap<String, i32> = FxHashMap::default();
    m.insert("a".to_string(), 1);
    println!("{}", m.get("a").copied().unwrap_or(0));
}
