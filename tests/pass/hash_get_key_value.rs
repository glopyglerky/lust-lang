// HashMap::get_key_value as std 1.99: both references borrow the map, and K and V keep the
// map's own lifetimes (ts-rust's interner: a stored &'static str key outlives the probe).
use std::collections::HashMap;
fn find<'a>(m: &'a HashMap<&'static str, u32>, s: &str) -> Option<(&'static str, u32)> {
    m.get_key_value(s).map(|(&k, &v)| (k, v))
}
fn main() {
    let mut m: HashMap<&'static str, u32> = HashMap::new();
    m.insert("a", 1); m.insert("bb", 2);
    let owned = String::from("bb");
    if let Some((&stored, &id)) = m.get_key_value(owned.as_str()) { println!("{stored} {id}"); }
    let kept: &'static str;
    { let probe = String::from("a"); kept = find(&m, &probe).unwrap().0; }
    println!("{kept} {:?} {:?}", m.get_key_value("zz"), find(&m, "a"));
    let mut n: HashMap<String, Vec<u8>> = HashMap::new();
    n.insert("k".into(), vec![1]);
    let (k, v) = n.get_key_value("k").unwrap();
    println!("{k} {v:?}");
    let e: HashMap<u8, u8> = HashMap::new(); println!("{:?}", e.get_key_value(&1));
}
