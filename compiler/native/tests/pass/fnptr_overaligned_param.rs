// Builtin fns as fn pointers with a parameter aligned over 16, which crosses C calls by
// pointer (`param_by_ptr`): std::mem::drop, Cell::set, Cell::replace and Cell::into_inner
// through `fn` values (their shims take the pointer and copy the value in), and a `&dyn Fn`
// call with such an argument.
use std::cell::Cell;

#[repr(align(32))]
#[derive(Debug, Clone, Copy, PartialEq)]
struct A32 {
    v: u64,
    w: u32,
}

struct D(A32);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop D {}", self.0.v);
    }
}

fn main() {
    let c = Cell::new(A32 { v: 1, w: 2 });
    let set: fn(&Cell<A32>, A32) = Cell::set;
    set(&c, A32 { v: 3, w: 4 });
    let rep: fn(&Cell<A32>, A32) -> A32 = Cell::replace;
    let old = rep(&c, A32 { v: 5, w: 6 });
    println!("{old:?} {:?}", c.get());
    let inner: fn(Cell<A32>) -> A32 = Cell::into_inner;
    println!("{:?}", inner(c));
    let d: fn(D) = std::mem::drop;
    d(D(A32 { v: 7, w: 1 }));
    let f: &dyn Fn(A32) -> u64 = &|a| a.v + a.w as u64;
    println!("{}", f(A32 { v: 5, w: 6 }));
    println!("{}", (0..4).map(|i| f(A32 { v: i, w: 1 })).sum::<u64>());
}
