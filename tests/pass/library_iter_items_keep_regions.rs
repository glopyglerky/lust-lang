// Items of a library or user iterator get their regions from its `next`
// signature: a `BTreeMap<&'static str, _>` key stays `'static` through a
// for loop, `map(..).collect()`, `last()`.
use std::collections::BTreeMap;
static NAMES: std::sync::Mutex<BTreeMap<&'static str, u64>> = std::sync::Mutex::new(BTreeMap::new());
pub fn report() -> Vec<(&'static str, u64)> {
    let names = NAMES.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    names.iter().map(|(k, v)| (*k, *v)).collect()
}
fn first(m: &BTreeMap<&'static str, u64>) -> Option<&'static str> {
    for (k, _) in m {
        return Some(*k);
    }
    None
}
fn last(m: &BTreeMap<&'static str, u64>) -> Option<&'static str> {
    m.keys().last().copied()
}
fn main() {
    NAMES.lock().unwrap().insert("a", 1);
    let mut m = BTreeMap::new();
    m.insert("b", 2u64);
    m.insert("c", 3);
    println!("{:?} {:?} {:?}", report(), first(&m), last(&m));
}
