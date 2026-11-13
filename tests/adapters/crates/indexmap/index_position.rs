// indexmap 2.14.0 `Index<usize>` and `IndexMut<usize>` (ts-rust
// files_parser.rs run_queued): the value at a position, next to the key
// index `map[&key]`.
use indexmap::IndexMap;

fn main() {
    let mut m: IndexMap<String, u32> = IndexMap::new();
    for (i, w) in ["b", "a", "c"].iter().enumerate() {
        m.insert(w.to_string(), i as u32 * 10);
    }
    println!("{} {} {}", m[0], m[2], m["a"]);
    m[1] += 5;
    let last = m.len() - 1;
    m[last] = 99;
    println!("{:?}", m);
}
