// `NonZero` fields in derived Copy, Eq, Hash, and Ord types (ts-rust
// `NodeArenaId(NonZeroU64)`): copies, set and map keys, ordering.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::num::{NonZero, NonZeroU32, NonZeroU64};

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeArenaId(NonZeroU64);
impl std::fmt::Debug for NodeArenaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NodeArenaId")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Snap {
    id: NodeArenaId,
    rev: u64,
    nz: Option<NonZero<u16>>,
    arr: [NonZeroU32; 2],
}

fn main() {
    let a = NodeArenaId(NonZeroU64::new(1).expect("nz"));
    let b = NodeArenaId(NonZeroU64::new(2).expect("nz"));
    let one = NonZeroU32::new(1).unwrap();
    let s = Snap { id: a, rev: 3, nz: NonZero::new(7u16), arr: [one, one] };
    let t = s;
    println!("{:?} {:?} {} {} {:?}", s, t, a < b, a == t.id, a.cmp(&b));
    let mut m: HashMap<NodeArenaId, u32> = HashMap::new();
    m.insert(a, 1);
    m.insert(b, 2);
    println!("{:?}", m.get(&b));
    let mut h: HashSet<Snap> = HashSet::new();
    h.insert(s);
    println!("{}", h.contains(&t));
    let mut bt = BTreeMap::new();
    bt.insert(b, "b");
    bt.insert(a, "a");
    println!("{:?}", bt.values().collect::<Vec<_>>());
}
