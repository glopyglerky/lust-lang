// The Debug builders through user impls: debug_struct, debug_tuple,
// debug_list, debug_set, debug_map (entry, key, value), their
// finish_non_exhaustive forms, and the pretty `{:#?}` form of each.
use std::fmt;

struct Point {
    x: i32,
    y: i32,
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point").field("x", &self.x).field("y", &self.y).finish()
    }
}

struct Opaque(u32);

impl fmt::Debug for Opaque {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Opaque").field("id", &self.0).finish_non_exhaustive()
    }
}

struct Empty;

impl fmt::Debug for Empty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Empty").finish_non_exhaustive()
    }
}

struct Pair(&'static str, Point);

impl fmt::Debug for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Pair").field(&self.0).field(&self.1).finish()
    }
}

struct Bag(Vec<u8>);

impl fmt::Debug for Bag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut l = f.debug_list();
        l.entry(&self.0[0]);
        l.entry(&"two");
        l.entry(&Point { x: 3, y: 4 });
        l.finish()
    }
}

struct Set3;

impl fmt::Debug for Set3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entry(&1).entry(&'c').finish_non_exhaustive()
    }
}

struct Map2;

impl fmt::Debug for Map2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entry(&"a", &1).key(&"b").value(&Point { x: 5, y: 6 }).finish()
    }
}

struct NoName;

impl fmt::Debug for NoName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("").field(&1).finish()
    }
}

fn main() {
    let p = Point { x: 1, y: -2 };
    println!("{:?} | {:?} | {:?}", p, Opaque(7), Empty);
    println!("{:?} | {:?} | {:?} | {:?} | {:?}", Pair("p", Point { x: 0, y: 0 }), Bag(vec![9]), Set3, Map2, NoName);
    println!("{:#?}", p);
    println!("{:#?}", Opaque(8));
    println!("{:#?}", Empty);
    println!("{:#?}", Pair("q", Point { x: 1, y: 1 }));
    println!("{:#?}", Bag(vec![1]));
    println!("{:#?}", Set3);
    println!("{:#?}", Map2);
    println!("{:#?}", NoName);
    println!("{:4?} {:<3?}", p, Pair("w", Point { x: 2, y: 3 }));
}
