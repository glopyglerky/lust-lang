// Written Index/IndexMut places (W-STD-C phase 2): a user
// container, BTreeMap by borrowed key, VecDeque by position;
// reads, writes, compound assignment, borrows of the place, and
// indexing through references and a Deref wrapper.
use std::collections::{BTreeMap, VecDeque};
use std::ops::{Deref, DerefMut, Index, IndexMut};

struct M(Vec<i32>);

impl Index<usize> for M {
    type Output = i32;
    fn index(&self, i: usize) -> &i32 {
        &self.0[i]
    }
}

impl IndexMut<usize> for M {
    fn index_mut(&mut self, i: usize) -> &mut i32 {
        &mut self.0[i]
    }
}

struct Wrap(VecDeque<i32>);

impl Deref for Wrap {
    type Target = VecDeque<i32>;
    fn deref(&self) -> &VecDeque<i32> {
        &self.0
    }
}

impl DerefMut for Wrap {
    fn deref_mut(&mut self) -> &mut VecDeque<i32> {
        &mut self.0
    }
}

fn main() {
    let mut m = M(vec![3, 4, 5]);
    m[0] += 10;
    let five = m[2];
    let thirteen = m[0];
    m[1] = five + thirteen;
    let r = &m[1];
    println!("{} {}", r, m[0]);

    let mut map = BTreeMap::from([(String::from("a"), 1), (String::from("b"), 2)]);
    *map.get_mut("a").unwrap() += 10;
    let v = &map["a"];
    println!("{} {}", v, map["b"]);

    let mut d = VecDeque::from([7, 8, 9]);
    let seven = d[0];
    d[1] += seven;
    let last = d.len() - 1;
    d[last] = 0;
    println!("{:?}", d);

    let rd = &d;
    println!("{}", rd[1]);

    let mut w = Wrap(VecDeque::from([5, 6]));
    w[0] += 1;
    println!("{} {:?}", w[1], w.0);
}
