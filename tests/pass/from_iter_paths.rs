// `Type::from_iter(arg)` by path for the std collections lust
// builds, in every spelling: bare and turbofish type paths, full
// std paths, `<T>::`, `<T as FromIterator<..>>::`, the item
// turbofish (the `IntoIterator` type), and `FromIterator::`
// with Self from the expected type. Capacities are `collect`'s,
// as rustc's `collect` is `from_iter`.
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

fn collect_vec<I: Iterator<Item = u32>>(i: I) -> Vec<u32> {
    Vec::from_iter(i)
}

fn refs<'a>(v: &'a Vec<u32>) -> Vec<&'a u32> {
    Vec::from_iter(v)
}

fn main() {
    let a: Vec<u32> = Vec::from_iter([1, 2, 3]);
    let b = Vec::<u32>::from_iter(0..3);
    let c: Vec<u32> = std::vec::Vec::from_iter(vec![7, 8]);
    let src = vec![9u32, 10];
    let d: Vec<u32> = Vec::from_iter(src.iter().copied());
    println!("{:?} {:?} {:?} {:?} caps {} {} {}", a, b, c, d, a.capacity(), b.capacity(), c.capacity());
    println!("{:?}", refs(&src));

    let e = Vec::<u32>::from_iter::<Vec<u32>>(vec![1, 2]);
    let f = Vec::<u32>::from_iter::<[u32; 2]>([3, 4]);
    let g = Vec::<u32>::from_iter::<_>([5]);
    println!("{:?} {:?} {:?}", e, f, g);

    let h: Vec<u32> = <Vec<u32>>::from_iter([6]);
    let i: Vec<u32> = <Vec<u32> as FromIterator<u32>>::from_iter([7, 8]);
    let j: Vec<u32> = <Vec<_> as FromIterator<_>>::from_iter([9u32]);
    let k: Vec<u32> = FromIterator::from_iter([10, 11]);
    println!("{:?} {:?} {:?} {:?}", h, i, j, k);
    println!("{:?}", collect_vec([1, 2].into_iter()));

    let s1: String = String::from_iter(['a', 'b']);
    let s2: String = String::from_iter(["cd", "e"]);
    let s3: String = <String as FromIterator<char>>::from_iter(['q']);
    let parts = vec!["f".to_string(), "g".to_string()];
    let s4: String = String::from_iter(parts);
    println!("{} {} {} {} caps {}", s1, s2, s3, s4, s1.capacity());

    let m: HashMap<u32, u32> = HashMap::from_iter([(1, 10), (2, 20), (1, 30)]);
    let s: HashSet<u32> = HashSet::from_iter([3, 1, 3]);
    let m2: HashMap<u32, u32> = std::collections::HashMap::from_iter((0..3).map(|i| (i, i * i)));
    println!("{} {:?} cap {} / {} {:?} cap {} / {:?}", m.len(), m.get(&1), m.capacity(), s.len(), s.contains(&1), s.capacity(), m2.get(&2));

    let bm: BTreeMap<u32, u32> = BTreeMap::from_iter([(2, 20), (1, 10), (2, 99)]);
    let bs: BTreeSet<u32> = BTreeSet::from_iter([3, 1, 3]);
    let bm2: BTreeMap<u32, u32> = BTreeMap::from_iter(vec![(5, 50)]);
    println!("{:?} {:?} {:?}", bm, bs, bm2);

    let q: VecDeque<u32> = VecDeque::from_iter([1, 2, 3]);
    let q2: VecDeque<u32> = VecDeque::from_iter(0..2);
    println!("{:?} cap {} {:?}", q, q.capacity(), q2);
}
