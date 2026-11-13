// A raw pointer field or map key names its pointee type (`*const Mapper`): every C unit
// that sees the pointer needs the pointee's forward typedef (tf-build: ts_goport's
// FxHashMap<*const Mapper, i32> made "';' expected (got 'A2480')" in a unit).
use std::collections::HashMap;
struct Mapper { name: String, n: u32 }
struct Loader { failures: HashMap<*const Mapper, i32>, seen: Vec<(*const Mapper, String)> }
fn main() {
    let a = Mapper { name: "a".into(), n: 1 };
    let b = Mapper { name: "b".into(), n: 2 };
    let mut l = Loader { failures: HashMap::new(), seen: Vec::new() };
    *l.failures.entry(&a).or_insert(0) += 1;
    *l.failures.entry(&b as *const Mapper).or_insert(0) += 2;
    *l.failures.entry(&a).or_insert(0) += 1;
    l.seen.push((&a, a.name.clone()));
    println!("{} {:?} {} {}", l.failures.len(), l.failures.get(&(&a as *const Mapper)), l.seen[0].1, a.n + b.n);
}
