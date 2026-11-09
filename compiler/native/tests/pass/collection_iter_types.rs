// The collection iterator structs are nameable by their std paths
// and keep the adapter set of the builtin iterators they wrap.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn count_deque(it: std::collections::vec_deque::Iter<'_, i32>) -> usize {
    it.count()
}

fn sum_keys(it: std::collections::btree_map::Keys<'_, i32, i32>) -> i32 {
    it.copied().sum()
}

fn main() {
    let mut d: VecDeque<i32> = VecDeque::from(vec![1, 2, 3]);
    println!("{} {}", count_deque(d.iter()), d.iter().copied().sum::<i32>());
    for x in d.iter_mut() {
        *x += 10;
    }
    println!("{:?}", d.iter().rev().collect::<Vec<_>>());
    let owned: Vec<i32> = d.into_iter().collect();
    println!("{:?}", owned);

    let mut m: BTreeMap<i32, i32> = BTreeMap::new();
    m.insert(2, 20);
    m.insert(1, 10);
    println!("{} {:?}", sum_keys(m.keys()), m.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>());
    for (_, v) in m.iter_mut() {
        *v += 1;
    }
    println!("{:?} {:?}", m.values().copied().collect::<Vec<_>>(), m.values_mut().count());
    println!("{:?}", m.values().collect::<BTreeSet<_>>().into_iter().rev().collect::<Vec<_>>());
    let mut dd: VecDeque<Vec<i32>> = VecDeque::new();
    dd.push_back(vec![1, 2]);
    dd.push_back(vec![3]);
    println!("{:?} {:?}", dd.iter().flatten().collect::<Vec<_>>(), dd.iter().flat_map(|v| v.iter()).collect::<Vec<_>>());
    println!("{:?} {:?}", m.keys().size_hint(), m.values().rposition(|v| *v == 21));
    let mut kit = m.keys();
    println!("{:?} {:?}", kit.next_back(), kit.next());
    println!("{:?}", m.values().chain(m.values()).collect::<Vec<_>>());
    let ks: Vec<i32> = m.clone().into_keys().collect();
    let vs: Vec<i32> = m.clone().into_values().collect();
    println!("{:?} {:?} {:?}", ks, vs, m.into_iter().collect::<Vec<_>>());

    let s: BTreeSet<i32> = vec![3, 1, 2].into_iter().collect();
    println!("{:?}", s.iter().rev().collect::<Vec<_>>());
    println!("{:?}", s.into_iter().collect::<Vec<_>>());
}
