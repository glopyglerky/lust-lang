// `std::collections::{BTreeMap, BTreeSet}` and `btree_map::Entry` are lust
// std source (library/alloc/src/collections.rs, W-STD-C): ordered
// iteration through the builtin adapters, lookups by `Borrow`, the entry
// API, retain, first/last, pop, split_off, append, from_iter, Debug.
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

fn main() {
    let mut m = BTreeMap::new();
    m.insert(3, "c");
    m.insert(1, "a");
    m.insert(2, "b");
    println!("{:?} {} {}", m, m.len(), m.is_empty());
    let keys: Vec<i32> = m.keys().copied().collect();
    let vals: Vec<&str> = m.values().copied().collect();
    println!("{:?} {:?}", keys, vals);
    println!("{:?} {:?} {:?}", m.get(&2), m.get(&9), m.get_key_value(&3));
    println!("{:?} {:?}", m.first_key_value(), m.last_key_value());
    println!("{:?} {:?}", m.insert(2, "B"), m.remove(&1));
    println!("{} {}", m.contains_key(&3), m.contains_key(&1));
    for (k, v) in m.iter_mut() {
        if *k > 2 {
            *v = "big";
        }
    }
    let pairs: Vec<(i32, &str)> = m.iter().map(|(k, v)| (*k, *v)).collect();
    println!("{:?} {}", pairs, m.iter().count());
    let rev: Vec<&i32> = m.keys().rev().collect();
    println!("{:?}", rev);

    let mut words: BTreeMap<String, u32> = BTreeMap::new();
    for w in ["pear", "apple", "pear", "fig", "pear", "apple"] {
        *words.entry(w.to_string()).or_insert(0) += 1;
    }
    println!("{:?} {:?}", words, words.get("pear"));
    words.entry(String::from("kiwi")).or_default();
    words.entry(String::from("fig")).and_modify(|n| *n += 10).or_insert(5);
    match words.entry(String::from("apple")) {
        Entry::Occupied(mut e) => {
            println!("occupied {} {}", e.key(), e.get());
            *e.get_mut() += 100;
        }
        Entry::Vacant(_) => println!("vacant"),
    }
    match words.entry(String::from("zed")) {
        Entry::Occupied(_) => println!("occupied"),
        Entry::Vacant(v) => {
            println!("vacant {}", v.key());
            println!("-> {}", v.insert(7));
        }
    }
    println!("{:#?}", words);
    words.retain(|k, v| k.len() > 3 && *v > 1);
    println!("{:?}", words);
    let total: u32 = words.values().sum();
    println!("{}", total);

    let mut a: BTreeMap<u8, char> = BTreeMap::new();
    for (i, c) in "hello".chars().enumerate() {
        a.insert(i as u8 * 2, c);
    }
    let hi = a.split_off(&4);
    println!("{:?} {:?}", a, hi);
    let mut b = BTreeMap::new();
    b.insert(0u8, 'X');
    b.insert(5, 'Y');
    a.append(&mut b);
    println!("{:?} {:?}", a, b);
    let (f, l) = (a.pop_first(), a.pop_last());
    println!("{:?} {:?} {:?}", f, l, a);
    let from: BTreeMap<&str, i32> = BTreeMap::from_iter(vec![("b", 1), ("a", 2), ("b", 3)].into_iter());
    println!("{:?} {}", from, from == from.clone());
    let into: Vec<(&str, i32)> = from.into_iter().collect();
    println!("{:?}", into);

    let mut s: BTreeSet<&str> = BTreeSet::new();
    for w in ["x", "a", "x", "m"] {
        s.insert(w);
    }
    println!("{:?} {} {} {:?}", s, s.contains("m"), s.contains("q"), s.iter().rev().collect::<Vec<_>>());
    println!("{:?} {:?} {}", s.first(), s.last(), s.len());
    let t: BTreeSet<i32> = BTreeSet::from_iter(vec![5, 1, 5, 3].into_iter());
    let u: BTreeSet<i32> = BTreeSet::from_iter(vec![1, 3].into_iter());
    println!("{:?} {} {} {}", t, u.is_subset(&t), t.is_superset(&u), t.is_disjoint(&u));
    let mut v = t.clone();
    v.retain(|x| *x != 3);
    let (r, t1) = (v.remove(&5), v.take(&1));
    println!("{:?} {} {:?}", v, r, t1);
    let doubled: Vec<i32> = t.iter().map(|x| x * 2).collect();
    println!("{:?} {:?}", doubled, t.iter().filter(|x| **x > 1).count());
}
