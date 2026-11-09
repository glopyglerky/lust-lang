// collect() whose target only a later use decides (Option::map(..).unwrap_or_default() into a
// field, a typed let after) into BTreeMap and BTreeSet: the FromIterator route, as for a known
// target (ts-rust effect/etscore.rs:808 into an IndexMap field).
use std::collections::{BTreeMap, HashMap};
struct Opts { aliases: HashMap<String, String>, sorted: BTreeMap<String, String> }
fn as_map(v: &BTreeMap<String, u32>) -> Option<&BTreeMap<String, u32>> { Some(v) }
fn as_str(v: &u32) -> Option<String> { (*v > 0).then(|| v.to_string()) }
fn first() {
    let src: BTreeMap<String, u32> = [("a".to_string(), 1), ("b".to_string(), 0)].into();
    let m: Option<&BTreeMap<String, u32>> = Some(&src);
    let o = Opts {
        aliases: m.and_then(as_map).map(|a| a.iter().filter_map(|(k, v)| Some((k.clone(), as_str(v)?.to_string()))).collect()).unwrap_or_default(),
        sorted: m.and_then(as_map).map(|a| a.iter().filter_map(|(k, v)| Some((k.clone(), as_str(v)?))).collect()).unwrap_or_default(),
    };
    println!("{:?} {:?}", o.aliases, o.sorted);
}

fn second() {
    let v = vec![("a".to_string(), 1u32)];
    let direct: BTreeMap<String, u32> = v.iter().cloned().collect();
    let late = Some(&v).map(|a| a.iter().cloned().collect()).unwrap_or_default();
    let typed: BTreeMap<String, u32> = late;
    println!("{:?} {:?}", direct, typed);
}
fn main() {
    first();
    second();
}
