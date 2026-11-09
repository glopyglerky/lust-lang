// `extend` and `from_iter` over a generic `IntoIterator` bound: the
// argument moves through its `into_iter` (a bound in the template, a
// written impl for a user collection), as a `for` head does.
use std::collections::HashMap;

fn add<T, I: IntoIterator<Item = T>>(v: &mut Vec<T>, i: I) {
    v.extend(i);
}

fn add_str<I: IntoIterator<Item = char>>(s: &mut String, i: I) {
    s.extend(i);
}

fn add_map<I: IntoIterator<Item = (i32, i32)>>(m: &mut HashMap<i32, i32>, i: I) {
    m.extend(i);
}

fn collect_all<T, I: IntoIterator<Item = T>>(i: I) -> Vec<T> {
    Vec::from_iter(i)
}

struct Coll;

struct CollIter {
    n: i32,
}

impl Iterator for CollIter {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.n < 3 {
            self.n += 1;
            Some(self.n)
        } else {
            None
        }
    }
}

impl IntoIterator for Coll {
    type Item = i32;
    type IntoIter = CollIter;
    fn into_iter(self) -> CollIter {
        CollIter { n: 0 }
    }
}

fn main() {
    let mut v = vec![1];
    add(&mut v, [2, 3]);
    add(&mut v, vec![4]);
    add(&mut v, 5..7);
    add(&mut v, Coll);
    v.extend(Coll);
    println!("{:?}", v);

    let mut s = String::from("a");
    add_str(&mut s, ['b', 'c']);
    add_str(&mut s, "de".chars());
    println!("{}", s);

    let mut m = HashMap::new();
    add_map(&mut m, [(1, 2), (3, 4)]);
    println!("{} {:?}", m.len(), m.get(&3));

    println!("{:?}", collect_all(Coll));
    println!("{:?}", collect_all([9, 8]));
}
