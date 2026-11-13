// A written PartialEq<Name> for String is not a key compare: BTreeMap<String, _> stays std's order (ts-rust ts_wasm host.rs:317).
use std::collections::BTreeMap;
struct Name(String);
impl PartialEq<String> for Name { fn eq(&self, o: &String) -> bool { self.0 == *o } }
impl PartialEq<Name> for String { fn eq(&self, o: &Name) -> bool { *self == o.0 } }
fn main() { let mut m: BTreeMap<String, u8> = BTreeMap::new(); m.insert("b".into(), 1); m.insert("a".into(), 2); println!("{:?} {}", m, String::from("x") == Name("x".into())); }
