// Entry closure forms with fn items, fn pointers, and references to
// closures, as std's FnOnce bound takes them.
use std::collections::HashMap;
fn mk() -> Vec<String> {
    vec![String::from("mk")]
}
fn key_len(k: &String) -> Vec<String> {
    vec![k.len().to_string()]
}
fn bump(v: &mut Vec<String>) {
    v.push(String::from("bumped"));
}
fn main() {
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    m.entry("a".into()).or_insert_with(mk).push("v0".into());
    m.entry("a".into()).or_insert_with(mk).push("v1".into());
    let f: fn() -> Vec<String> = Vec::new;
    m.entry("b".into()).or_insert_with(f);
    m.entry("ccc".into()).or_insert_with_key(key_len);
    m.entry("a".into()).and_modify(bump);
    let mut n = 0;
    let mut g = || {
        n += 1;
        vec![]
    };
    m.entry("d".into()).or_insert_with(&mut g);
    m.entry("d".into()).or_insert_with(&mut g);
    println!("calls {}", n);
    let mut ks: Vec<_> = m.into_iter().collect();
    ks.sort();
    println!("{:?}", ks);
}
