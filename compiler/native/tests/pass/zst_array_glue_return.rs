// A zero-sized type whose C struct holds an array ([T; 0], [No; 4], or an
// aligned empty struct) is returned through an out-pointer: tcc returned it
// in a register and the caller wrote a byte over its own frame (a segfault
// from derive(Default) on a Cell<[No; 0]> field).
use std::cell::Cell;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct No;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(align(8))]
struct Al8;

#[derive(Default)]
struct S {
    f: Cell<[No; 0]>,
}

#[derive(Default, Clone)]
struct T {
    a: u32,
    f: Cell<[No; 4]>,
    g: Option<[u8; 0]>,
    h: (Al8, [u64; 0]),
}

#[derive(Default, Clone, Debug)]
struct W<X> {
    x: X,
    n: u8,
}

fn mk_al(k: u8) -> Al8 {
    println!("mk {k}");
    Al8
}

fn mk_pair(k: u8) -> (Cell<[No; 2]>, Al8) {
    println!("pair {k}");
    (Cell::new([No; 2]), Al8)
}

fn main() {
    let s = S::default();
    println!("{}", std::mem::size_of_val(&s) > 0);
    let t = T::default();
    let u = t.clone();
    println!("{} {:?} {:?} {:?}", u.a, u.f.get(), u.g, u.h);
    let a = mk_al(1);
    let (c, b) = mk_pair(2);
    println!("{:?} {:?} {:?}", a, b, c.get());
    let w: W<Cell<[No; 0]>> = W::default();
    let w2 = w.clone();
    println!("{} {:?}", w2.n, w2.x.get());
    println!("{} {}", std::mem::size_of::<T>(), std::mem::align_of::<T>());
}
