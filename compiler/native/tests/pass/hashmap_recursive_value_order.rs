// A recursive enum reached through a HashMap value: the table struct holds
// only its core and builder, so `Any` must not wait for the `(String, Any)`
// element tuple (tcc said "field 'f1' has incomplete type"; tf-p4 q2/u6).
use std::collections::HashMap;

enum Any {
    Obj(HashMap<String, Any>),
    Num(u32),
}

fn lookup<'c>(cfg: &'c HashMap<String, Any>, path: &str) -> Option<&'c Any> {
    let mut cur: Option<&'c Any> = None;
    for part in path.split('.') {
        let m = match cur {
            None => cfg,
            Some(Any::Obj(m)) => m,
            Some(_) => return None,
        };
        cur = Some(m.get(part)?);
    }
    cur
}

fn main() {
    let mut h = HashMap::new();
    h.insert("a".to_string(), Any::Num(1));
    if let Some(Any::Num(n)) = h.get("a") {
        println!("{n}");
    }
    let inner = HashMap::from([("b".to_string(), Any::Num(7))]);
    h.insert("o".to_string(), Any::Obj(inner));
    match lookup(&h, "o.b") {
        Some(Any::Num(n)) => println!("o.b = {n}"),
        _ => println!("o.b missing"),
    }
    println!("{}", lookup(&h, "a.b").is_none());
    let e = h.entry("z".to_string()).or_insert(Any::Num(3));
    if let Any::Num(n) = e {
        *n += 1;
    }
    let mut keys: Vec<&String> = h.keys().collect();
    keys.sort();
    println!("{keys:?}");
}
