// expect: L0640
// Valid Rust; a hash table's `clone_from` reuses its bucket array
// in ways the clonefrom glue does not model yet.
fn main() {
    let mut m = std::collections::HashMap::from([(1, "a")]);
    m.clone_from(&std::collections::HashMap::from([(2, "b")]));
    println!("{:?}", m);
}
