// The by-value array forms (fx-array-iter): `for`, `into_iter`,
// `collect`, `from_iter`, `HashMap::from`, `HashSet::from`,
// `extend`, flatten, generic `IntoIterator` bounds, the written
// `std::array::IntoIter` type, and arrays longer than 32. The
// by-reference forms (`(&arr).into_iter()`, `arr.iter()`) stay by
// reference and leave the array usable.
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

type StrSet = HashSet<String>;
type StrMap = HashMap<String, String>;

fn count<I: IntoIterator>(x: I) -> usize {
    x.into_iter().count()
}

fn first<I: IntoIterator>(x: I) -> Option<I::Item> {
    x.into_iter().next()
}

fn named(it: std::array::IntoIter<String, 2>) -> Vec<String> {
    it.collect()
}

fn main() {
    for s in [String::from("a"), String::from("bb")] {
        println!("{}", s);
    }
    let v: Vec<String> = [String::from("x"), String::from("y")].into_iter().collect();
    println!("{:?}", v);
    let set: HashSet<String> = HashSet::from_iter([String::from("s1"), String::from("s2"), String::from("s1")]);
    println!("set {} {}", set.len(), set.contains("s2"));
    let set2 = HashSet::from([String::from("a"), String::from("b")]);
    println!("set2 {}", set2.len());
    let m = HashMap::from([(String::from("k1"), String::from("v1")), (String::from("k2"), String::from("v2"))]);
    println!("map {} {:?}", m.len(), m.get("k2"));
    let m2: HashMap<String, Vec<u32>> = HashMap::from_iter([(String::from("a"), vec![1])]);
    println!("map2 {:?}", m2.get("a"));
    let mut v3: Vec<String> = Vec::new();
    v3.extend([String::from("e1"), String::from("e2")]);
    println!("{:?}", v3);
    let mut m3: HashMap<String, i32> = HashMap::new();
    m3.extend([(String::from("x"), 1), (String::from("y"), 2)]);
    println!("m3 {} {:?}", m3.len(), m3.get("y"));
    // The alias forms, as ts-rust's `FxHashSet::from_iter([s])` writes.
    let aset = StrSet::from_iter([String::from("only")]);
    println!("aset {}", aset.len());
    let amap = StrMap::from([(String::from("k"), String::from("v"))]);
    println!("amap {:?}", amap.get("k"));
    let sc: String = [String::from("a"), String::from("b")].into_iter().collect();
    println!("{}", sc);
    let bm = BTreeMap::from([(String::from("b"), 2), (String::from("a"), 1)]);
    println!("{:?}", bm);
    let bs = BTreeSet::from([String::from("y"), String::from("x")]);
    println!("{:?}", bs);
    let q: VecDeque<String> = VecDeque::from([String::from("q1"), String::from("q2")]);
    println!("{:?}", q);
    // By reference first, by value after: the array survives the
    // reference iterations.
    let mut arr = [String::from("p"), String::from("q")];
    for s in (&arr).into_iter() {
        println!("ref {}", s);
    }
    for s in arr.iter() {
        println!("iter {}", s);
    }
    for s in (&mut arr).into_iter() {
        s.push('!');
    }
    println!("still {:?}", arr);
    for s in arr {
        println!("own {}", s);
    }
    // Generic bounds and the written type name.
    println!("{} {}", count([String::from("g")]), count([1, 2, 3]));
    println!("{:?}", first([String::from("f1"), String::from("f2")]));
    println!("{:?}", named([String::from("n1"), String::from("n2")].into_iter()));
    // Flatten over arrays of arrays.
    let f: Vec<String> = [[String::from("a")], [String::from("b")]].into_iter().flatten().collect();
    println!("{:?}", f);
    let fm: Vec<String> = ["a", "b"].into_iter().flat_map(|s| [s.to_string(), s.to_string()]).collect();
    println!("{:?}", fm);
    // Longer than the old 32-element for-loop unroll.
    let big: [Vec<u32>; 35] = [
        vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1],
        vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1],
        vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1], vec![1],
    ];
    let mut n = 0;
    for v in big {
        n += v.len();
    }
    println!("big {}", n);
}
