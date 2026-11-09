// HashMap::retain, HashSet::retain, HashMap::remove_entry: hashbrown's
// order (one predicate call per element in iteration order), drops of
// rejected elements before the next call, capacity kept, and the
// erased buckets reused by later inserts.
use std::collections::{HashMap as HM, HashSet as HS};
use std::hash::{BuildHasherDefault, DefaultHasher};
type HashMap<K, V> = HM<K, V, BuildHasherDefault<DefaultHasher>>;
type HashSet<K> = HS<K, BuildHasherDefault<DefaultHasher>>;

#[derive(Debug)]
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D{}", self.0);
    }
}

struct Index {
    aliases: HashMap<u32, u32>,
}
impl Index {
    fn drop_file(&mut self, file: u32) {
        self.aliases.retain(|n, _| n % 4 != file);
    }
}

fn keep(k: &u32, v: &mut u32) -> bool {
    *v += 1;
    k % 3 != 0
}

fn prune<K: std::fmt::Debug, V, S>(m: &mut HM<K, V, S>, every: usize) {
    let mut i = 0;
    m.retain(|k, _| {
        i += 1;
        println!("see {:?}", k);
        i % every != 0
    });
}

fn main() {
    let mut m: HashMap<u32, D> = HashMap::default();
    for i in 0..20 {
        m.insert(i, D(i));
    }
    let mut calls = 0;
    m.retain(|k, v| {
        calls += 1;
        v.0 += 100;
        k % 3 != 0
    });
    println!("calls {} len {} cap {}", calls, m.len(), m.capacity());
    for i in 100..110 {
        m.insert(i, D(i));
    }
    println!("{:?}", m.keys().collect::<Vec<_>>());
    let e = m.remove_entry(&5);
    println!("{:?}", e.as_ref().map(|(k, v)| (*k, v.0)));
    println!("{}", m.remove_entry(&5).is_none());
    drop(e);

    let mut s: HashSet<String> = (0..12).map(|i| format!("s{i}")).collect();
    let dead: HashSet<&str> = ["s1", "s4"].into_iter().collect();
    s.retain(|x| !dead.contains(x.as_str()));
    println!("{:?}", s);

    let mut by_str: HashMap<&str, Vec<u8>> = HashMap::default();
    by_str.insert("a", vec![1]);
    by_str.insert("b", vec![]);
    by_str.retain(|k, v| k.len() == 1 && !v.is_empty());
    println!("{:?}", by_str);
    let mut owned: HashMap<String, u32> = HashMap::default();
    owned.insert("x".to_string(), 1);
    println!("{:?} {:?}", owned.remove_entry("x"), owned.remove_entry("x"));

    let mut ix = Index { aliases: (0..16).map(|i| (i, i * 2)).collect() };
    ix.drop_file(1);
    ix.drop_file(2);
    ix.aliases.retain(keep);
    println!("{:?}", ix.aliases);
    prune(&mut ix.aliases, 2);
    let g = D(999);
    ix.aliases.retain(move |k, _| g.0 > *k + 990);
    println!("after move closure {:?}", ix.aliases);
    m.retain(|_, _| false);
    println!("len {} cap {}", m.len(), m.capacity());
    m.insert(7, D(7));
    println!("end");
}
