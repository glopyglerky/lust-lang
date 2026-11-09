// Collecting cells into a Vec needs their layout (RawVec's first capacity): HashMap
// into_values, iterator chains and in-place collect over Cell, RefCell and rc::Weak
// elements. Their size_of and Option niches match std.
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

struct S1 {
    a: u8,
    b: u64,
}

fn main() {
    let mut m: HashMap<u32, Cell<u64>> = HashMap::new();
    m.insert(1, Cell::new(5));
    m.insert(2, Cell::new(7));
    let mut v: Vec<Cell<u64>> = m.into_values().collect();
    v.sort_by_key(|c| c.get());
    println!("{:?} {}", v.iter().map(|c| c.get()).collect::<Vec<_>>(), v.capacity());

    static A: u8 = 1;
    let mut r: HashMap<u32, Cell<&u8>> = HashMap::new();
    r.insert(1, Cell::new(&A));
    let rs: Vec<_> = r.into_values().collect();
    println!("{} {}", rs.len(), rs.capacity());

    let bytes: Vec<Cell<u8>> = (0..3u8).map(|x| Cell::new(x)).collect();
    let cells: Vec<RefCell<String>> = vec!["a", "b"].into_iter().map(|s| RefCell::new(s.to_string())).collect();
    cells[1].borrow_mut().push('!');
    println!("{} {} {:?}", bytes.len(), bytes.capacity(), cells.iter().map(|c| c.borrow().clone()).collect::<Vec<_>>());

    let rc = Rc::new(3);
    let weaks: Vec<Weak<i32>> = (0..2).map(|_| Rc::downgrade(&rc)).collect();
    println!("{} {}", weaks.len(), weaks.iter().filter_map(|w| w.upgrade()).map(|x| *x).sum::<i32>());

    println!("{} {}", std::mem::size_of::<Cell<u8>>(), std::mem::size_of::<Option<Cell<&u8>>>());
    println!("{} {}", std::mem::size_of::<RefCell<u8>>(), std::mem::align_of::<RefCell<(u8, u32)>>());
    println!("{} {}", std::mem::size_of::<RefCell<S1>>(), std::mem::size_of::<Option<RefCell<bool>>>());
    println!("{} {}", std::mem::size_of::<Weak<u8>>(), std::mem::size_of::<Option<Weak<String>>>());
    println!("{}", std::mem::size_of::<Cell<[u16; 3]>>());
    let k = RefCell::new(S1 { a: 1, b: 2 });
    let g = k.borrow();
    println!("{} {}", std::mem::size_of_val(&g), g.a as u64 + g.b);
}
