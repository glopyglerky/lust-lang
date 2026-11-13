// `std::cmp::Ordering` as a plain fieldless enum: variant paths
// (four-segment and after `use`), match, copy, equality, fields,
// consts, and layout.
use std::cmp::Ordering;

fn sign(a: u8, b: u8) -> i32 {
    match a.cmp(&b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

struct S {
    o: Ordering,
    n: u8,
}

const O: Ordering = std::cmp::Ordering::Greater;

fn main() {
    println!("{} {} {}", sign(1, 2), sign(2, 2), sign(3, 2));
    let a = std::cmp::Ordering::Less;
    let b = a;
    println!("{} {:?}", a == b, a);
    let s = S { o: 1u8.cmp(&2u8), n: 3 };
    println!("{:?} {}", s.o, s.n);
    println!("{:?}", O);
    println!("{} {}", std::mem::size_of::<Ordering>(), std::mem::size_of::<Option<Ordering>>());
}
