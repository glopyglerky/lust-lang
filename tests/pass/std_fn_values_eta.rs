// Std fns used as values that lust has no item value for (TRIAGE R15,
// L9001): `Vec::new`, `Arc::new`, `Rc::downgrade`, `Ord::cmp`,
// `HashMap::default`, ... They eta-expand to a closure at parse time,
// which behaves as the fn item does.
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::time::Duration;

type FxMap<K, V> = HashMap<K, V>;

fn apply<T, R>(f: impl Fn(T) -> R, x: T) -> R {
    f(x)
}

fn make<R>(f: fn() -> R) -> R {
    f()
}

fn main() {
    let mut m: HashMap<u8, Vec<u8>> = HashMap::new();
    m.entry(1).or_insert_with(Vec::new).push(2);
    m.entry(1).or_insert_with(Vec::new).push(3);
    println!("{:?}", m);
    let o: Option<String> = None;
    println!("[{}]", o.unwrap_or_else(String::new));
    let mut fx: Option<FxMap<u8, u8>> = None;
    fx.get_or_insert_with(FxMap::default).insert(1, 2);
    println!("{:?}", fx);
    let mut hs: Option<HashSet<u8>> = None;
    hs.get_or_insert_with(HashSet::default).insert(4);
    println!("{:?}", hs);
    let b: BTreeMap<u8, u8> = make(BTreeMap::new);
    let d: VecDeque<u8> = make(VecDeque::new);
    println!("{:?} {:?}", b, d);
    let a: Vec<Arc<u8>> = vec![3u8, 4].into_iter().map(Arc::new).collect();
    println!("{:?}", a);
    let r = Rc::new(5);
    let w: Option<Weak<i32>> = Some(&r).map(Rc::downgrade);
    println!("{:?} {}", w.and_then(|w| w.upgrade()), Rc::strong_count(&r));
    let c = Some(&r).map(Rc::clone);
    println!("{:?} {}", c, apply(Rc::strong_count, &r));
    let none: Weak<u8> = None.unwrap_or_else(Weak::new);
    println!("{:?}", none.upgrade());
    println!("{:?}", Some(65u32).and_then(char::from_u32));
    let mut v = vec![3, 1, 2];
    v.sort_by(Ord::cmp);
    let mut s = vec!["b", "a", "c"];
    s.sort_by(Ord::cmp);
    println!("{:?} {:?} {:?}", v, s, apply(|(x, y)| Ord::max(x, y), (4, 9)));
    let ds: Vec<Duration> = vec![1u64, 2].into_iter().map(Duration::from_millis).collect();
    println!("{:?}", ds);
    let sleep: fn(Duration) = std::thread::sleep;
    sleep(Duration::from_millis(1));
    let mut slot = vec![1u8];
    let old = apply(std::mem::take, &mut slot);
    println!("{:?} {:?}", old, slot);
}
