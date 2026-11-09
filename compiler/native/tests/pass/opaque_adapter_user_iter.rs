// An `impl Iterator` whose hidden type is a builtin adapter over another
// opaque or a type param that becomes a user iterator, a `from_fn`, or a
// btree iterator. The caller's reveal once had the bare iterator where
// the defining body had its `ByNext` wrapper (or the btree iterator's
// inner iterator): "iterator expansion: no item type" or a C type clash.
use std::collections::BTreeMap;

struct Cnt(u8);

impl Iterator for Cnt {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if self.0 < 3 {
            self.0 += 1;
            Some(self.0)
        } else {
            None
        }
    }
}

fn mk() -> impl Iterator<Item = u8> {
    Cnt(0)
}

fn bump<I: Iterator<Item = u8>>(it: I) -> impl Iterator<Item = u8> {
    it.map(|v| v + 10)
}

fn twice<I: Iterator<Item = i64>>(it: I) -> impl Iterator<Item = i64> {
    it.map(|v| v * 2)
}

fn mapped() -> impl Iterator<Item = u8> {
    mk().map(|v| v * 2)
}

fn zipped() -> impl Iterator<Item = (u8, u8)> {
    (0..3u8).zip(mk())
}

fn chained() -> impl Iterator<Item = u8> {
    (0..1u8).chain(mk())
}

fn from_closure(x: i64) -> impl Iterator<Item = i64> {
    let mut n = 0;
    std::iter::from_fn(move || {
        n += 1;
        if n <= 2 { Some(x * n) } else { None }
    })
}

fn from_closure_mapped() -> impl Iterator<Item = i64> {
    from_closure(-3).map(|v| v - 1)
}

fn keys(m: &BTreeMap<u8, u8>) -> impl Iterator<Item = &u8> + '_ {
    m.keys()
}

fn keys_mapped(m: &BTreeMap<u8, u8>) -> impl Iterator<Item = u8> + '_ {
    keys(m).map(|k| k + 1)
}

fn main() {
    println!("{:?}", bump(Cnt(0)).collect::<Vec<_>>());
    println!("{:?}", mapped().collect::<Vec<_>>());
    println!("{:?}", zipped().collect::<Vec<_>>());
    println!("{:?}", chained().collect::<Vec<_>>());
    println!("{:?}", from_closure_mapped().collect::<Vec<_>>());
    for v in twice(from_closure_mapped()) {
        println!("{v}");
    }
    let mut m = BTreeMap::new();
    m.insert(1, 2);
    m.insert(3, 4);
    println!("{:?}", keys_mapped(&m).collect::<Vec<_>>());
    println!("{}", mapped().filter(|v| *v > 2).count());
}
