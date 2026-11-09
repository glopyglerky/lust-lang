// feature: ext.indexmap
// form: IndexMap insert, lookup, and insertion-order iteration
// cargo-dep: indexmap = "=2.14.0"
use indexmap::IndexMap;
fn main() {
    let mut m = IndexMap::new();
    m.insert("b", 1);
    m.insert("a", 2);
    m.insert("b", 3);
    for (k, v) in &m {
        println!("{} {}", k, v);
    }
    println!("{:?} {:?}", m.get("a"), m.get_index_of("b"));
}
