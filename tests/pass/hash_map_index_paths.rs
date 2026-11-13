// `map[&q]` through Rc, Box, struct fields, a returned reference, a custom
// hasher, a two-phase `insert` argument, and as a match scrutinee (W-STD-B).
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, DefaultHasher};
use std::rc::Rc;

struct Db {
    names: HashMap<u32, String>,
}

fn first(db: &Db) -> &str {
    &db.names[&1]
}

fn main() {
    let rc = Rc::new(HashMap::from([("k", 7u8)]));
    let bx = Box::new(HashMap::from([(1u8, 'q')]));
    println!("{} {}", rc["k"], bx[&1]);
    let mut names = HashMap::new();
    names.insert(1, String::from("one"));
    let db = Db { names };
    println!("{} {}", first(&db), db.names[&1].to_uppercase());
    let mut c: HashMap<u8, u8, BuildHasherDefault<DefaultHasher>> = HashMap::default();
    c.insert(3, 4);
    println!("{}", c[&3]);
    let mut m = HashMap::from([(0u8, 10u8)]);
    m.insert(1, m[&0] + 1);
    println!("{}", m[&1]);
    match m[&1] {
        11 => println!("eleven"),
        _ => println!("other"),
    }
    if let Some(x) = Some(m[&0]) {
        println!("{}", x);
    }
    let keys = vec![0u8, 1];
    let vals: Vec<u8> = keys.iter().map(|k| m[k]).collect();
    println!("{:?}", vals);
}
