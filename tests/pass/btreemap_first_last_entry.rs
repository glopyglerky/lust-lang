// BTreeMap::first_entry and last_entry (ts-rust program.rs run_bind_link):
// the smallest and largest key's OccupiedEntry, None on an empty map.
use std::collections::BTreeMap;

fn main() {
    let mut waiting: BTreeMap<u32, String> = BTreeMap::new();
    println!("{}", waiting.first_entry().is_none());
    for k in [5, 2, 9, 7] {
        waiting.insert(k, format!("v{k}"));
    }
    while let Some(entry) = waiting.first_entry() {
        if *entry.key() > 6 {
            break;
        }
        let (k, v) = entry.remove_entry();
        println!("{k} {v}");
    }
    if let Some(mut e) = waiting.last_entry() {
        e.get_mut().push('!');
        println!("{} {}", e.key(), e.get());
    }
    if let Some(e) = waiting.last_entry() {
        println!("{}", e.remove());
    }
    println!("{:?}", waiting);
}
