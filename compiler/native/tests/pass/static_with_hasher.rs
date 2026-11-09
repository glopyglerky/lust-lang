// `HashMap::with_hasher` and `HashSet::with_hasher` are const fns: statics
// and const fns build empty tables around a hasher value (ts-rust POOL,
// PerFileMap::new); iteration order and capacity follow std.
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use std::sync::{Mutex, RwLock};

#[derive(Default)]
struct FxHasher(u64);
impl Hasher for FxHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0.rotate_left(5) ^ b as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
        }
    }
}
#[derive(Clone, Copy, Default)]
struct FxBuild;
impl BuildHasher for FxBuild {
    type Hasher = FxHasher;
    fn build_hasher(&self) -> FxHasher {
        FxHasher(0)
    }
}
type FxMap<K, V> = HashMap<K, V, FxBuild>;
type FxSet<T> = HashSet<T, FxBuild>;
type SipMap = HashMap<u32, u32, BuildHasherDefault<DefaultHasher>>;

struct Pool {
    free: Vec<u32>,
    cols: FxMap<u32, String>,
    seen: FxSet<u64>,
}
impl Pool {
    const fn new() -> Self {
        Pool { free: Vec::new(), cols: FxMap::with_hasher(FxBuild), seen: HashSet::with_hasher(FxBuild) }
    }
}
static POOL: Mutex<Pool> = Mutex::new(Pool::new());
static MAP: RwLock<FxMap<&'static str, u8>> = RwLock::new(FxMap::with_hasher(FxBuild));
static SIP: Mutex<SipMap> = Mutex::new(SipMap::with_hasher(BuildHasherDefault::new()));
thread_local! {
    static SEEN: RefCell<FxSet<u32>> = const { RefCell::new(FxSet::with_hasher(FxBuild)) };
}

fn main() {
    {
        let mut p = POOL.lock().unwrap();
        p.free.push(1);
        for i in 0..20u32 {
            p.cols.insert(i, format!("v{i}"));
        }
        p.seen.insert(7);
        let mut ks: Vec<_> = p.cols.keys().copied().collect();
        ks.sort();
        println!("{} {} {:?} {}", p.free.len(), p.cols.len(), &ks[..3], p.seen.contains(&7));
        println!("{:?}", p.cols.iter().map(|(k, _)| *k).collect::<Vec<_>>());
        println!("{}", p.cols.capacity());
    }
    MAP.write().unwrap().insert("a", 1);
    println!("{:?} {}", MAP.read().unwrap().get("a"), MAP.read().unwrap().capacity());
    SIP.lock().unwrap().insert(3, 4);
    println!("{:?}", SIP.lock().unwrap().get(&3));
    SEEN.with(|s| {
        s.borrow_mut().insert(5);
        println!("{} {}", s.borrow().len(), s.borrow().capacity());
    });
    let fresh: FxMap<u32, String> = Pool::new().cols;
    println!("{} {}", fresh.len(), fresh.capacity());
}
