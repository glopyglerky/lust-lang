// indexmap 2.14.0: table churn through every control-byte path the
// adapter's group scans and bucket loops take (inserts that grow from the
// singleton, swap_remove and shift_remove tombstones, rehash in place,
// retain, values_mut, clear and reuse, tables smaller than one group),
// and the index panic of a missing key. Capacities and orders are the
// real crate's.
use indexmap::IndexMap;

fn main() {
    let mut m: IndexMap<u64, u64> = IndexMap::new();
    for round in 0..4u64 {
        for i in 0..300u64 {
            m.insert(i * 7919 % 1000 + round, i);
        }
        for i in (0..300u64).step_by(3) {
            m.swap_remove(&(i * 7919 % 1000 + round));
        }
        for i in (1..300u64).step_by(5) {
            m.shift_remove(&(i * 7919 % 1000 + round));
        }
        println!("round {round}: len {} cap>=len {} first {:?} last {:?}", m.len(), m.capacity() >= m.len(), m.first(), m.last());
    }
    m.retain(|k, v| (k + *v) % 3 != 0);
    for v in m.values_mut() {
        *v += 1;
    }
    let s: u64 = m.iter().map(|(k, v)| k ^ v).sum();
    println!("retain: len {} sum {s} get {:?} {:?}", m.len(), m.get(&1), m.get_index_of(&500));
    m.clear();
    for i in 0..5u64 {
        m.insert(i, i * i);
    }
    println!("small: {:?} {:?}", m, m.get(&4));
    let mut tiny: IndexMap<&str, u8> = IndexMap::with_capacity(3);
    tiny.insert("a", 1);
    tiny.insert("b", 2);
    tiny.swap_remove("a");
    tiny.insert("c", 3);
    println!("tiny: {:?} {}", tiny, tiny.contains_key("a"));
    let r = std::panic::catch_unwind(|| tiny["zz"]);
    println!("missing key panics: {}", r.is_err());
}
