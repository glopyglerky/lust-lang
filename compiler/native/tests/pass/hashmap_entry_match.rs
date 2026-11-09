// A match on `HashMap::entry` by its variants, with the payload bound by
// value as an `OccupiedEntry` or `VacantEntry` (ts-rust checker/flow_skip.rs
// `flow_skip_test_worker`): full std paths, an imported `Entry`, `if let`,
// `let else`, wildcard payloads, and a probe key with a destructor.
use std::collections::HashMap;
use std::collections::hash_map::{self, Entry};
#[derive(Debug, PartialEq, Eq, Hash)]
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D{}", self.0);
    }
}
#[derive(Clone, Copy, Debug)]
struct Count {
    countdown: u32,
    gap: u32,
}
enum FileState {
    Counting(Count),
    Unsupported,
}
fn count(files: &mut HashMap<u32, FileState>, file: u32, ok: bool) -> Option<u32> {
    let c = match files.entry(file) {
        std::collections::hash_map::Entry::Occupied(entry) => match entry.into_mut() {
            FileState::Unsupported => return None,
            FileState::Counting(count) => Some(*count),
        },
        std::collections::hash_map::Entry::Vacant(entry) => {
            if !ok {
                entry.insert(FileState::Unsupported);
                return None;
            }
            let count = Count { countdown: 3, gap: 3 };
            entry.insert(FileState::Counting(count));
            Some(count)
        }
    }?;
    Some(c.countdown + c.gap)
}
fn main() {
    let mut files = HashMap::new();
    println!("{:?} {:?} {:?} {:?}", count(&mut files, 1, true), count(&mut files, 1, false), count(&mut files, 2, false), count(&mut files, 2, true));
    let mut m: HashMap<D, u32> = HashMap::new();
    m.insert(D(1), 10);
    for k in [1, 2, 1, 3] {
        let r = match m.entry(D(k)) {
            Entry::Occupied(mut e) => {
                *e.get_mut() += 1;
                println!("occupied {:?} {}", e.key(), e.get());
                0
            }
            Entry::Vacant(v) => {
                println!("vacant {:?}", v.key());
                *v.insert(k * 100)
            }
        };
        println!("r {r}");
    }
    if let hash_map::Entry::Occupied(e) = m.entry(D(3)) {
        println!("removed {}", e.remove());
    }
    let Entry::Vacant(v) = m.entry(D(4)) else { return };
    let o = v.insert_entry(7);
    println!("{}", o.get());
    let n = match m.entry(D(9)) {
        Entry::Occupied(_) => 1,
        Entry::Vacant(_) => 2,
    };
    println!("{n} {}", m.len());
    let mut all: Vec<_> = m.drain().collect();
    all.sort_by_key(|(k, _)| k.0);
    drop(all);
}
