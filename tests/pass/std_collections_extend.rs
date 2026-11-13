// Collection Extend, From<[_; N]>, and VecDeque::from_iter
// (W-STD-C phase 2): insert-loop extend (an old key stays, the
// new value wins, replaced values drop in order), bulk From (the
// last of equal keys stays), and VecDeque collect capacity.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
impl Clone for D {
    fn clone(&self) -> D {
        println!("clone {}", self.0);
        D(self.0)
    }
}

fn main() {
    let mut m: BTreeMap<i32, D> = BTreeMap::new();
    m.insert(1, D(1));
    m.extend(vec![(1, D(2)), (2, D(3)), (1, D(4))]);
    println!("map len {}", m.len());
    drop(m);

    let f = BTreeMap::from([(3, "c"), (1, "a"), (3, "c2"), (2, "b")]);
    println!("{:?}", f);
    let fs = BTreeSet::from([5, 3, 5, 1, 3]);
    println!("{:?}", fs);

    let mut s = BTreeSet::from([1, 2]);
    s.extend(vec![2, 3]);
    s.extend(3..5);
    println!("{:?}", s);

    let pairs = vec![(1, 10), (2, 20)];
    let mut m2: BTreeMap<i32, i32> = BTreeMap::new();
    m2.extend(pairs);
    println!("{:?}", m2);

    let mut d: VecDeque<i32> = (0..5).collect();
    println!("{:?} cap {}", d, d.capacity());
    d.pop_front();
    d.extend(vec![9, 8]);
    d.extend((0..10).collect::<Vec<_>>());
    println!("{:?} cap {}", d, d.capacity());
    let e: VecDeque<i32> = (0..9).filter(|x| x % 2 == 0).collect();
    println!("{:?} cap {}", e, e.capacity());
    let g = VecDeque::from([7, 8, 9]);
    println!("{:?} cap {}", g, g.capacity());

    let mut dd: VecDeque<D> = VecDeque::new();
    dd.extend(vec![D(1), D(2)]);
    let mut di: VecDeque<i32> = VecDeque::new();
    di.extend(vec![3]);
    println!("dd len {} di {:?}", dd.len(), di);
    drop(dd);
}
