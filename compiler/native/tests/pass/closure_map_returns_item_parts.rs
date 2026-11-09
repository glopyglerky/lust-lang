// `map` closures return borrows of their items (the kernels' `IndexMap`
// shapes: `|(k, v)| (k, v)`, `|e| &e.1`).
fn main() {
    let entries: Vec<(String, u32)> = vec![(String::from("a"), 1), (String::from("b"), 2)];
    let pairs: Vec<(&String, &u32)> = entries.iter().map(|(k, v)| (k, v)).collect();
    let keys: Vec<&String> = entries.iter().map(|(k, _)| k).collect();
    let second: Vec<&u32> = entries.iter().map(|e| &e.1).collect();
    println!("{:?} {:?} {:?}", pairs, keys, second);
}
