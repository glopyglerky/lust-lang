// expect: L0612
// rustc E0277: `Cell::take` needs `T: Default`.
use std::cell::Cell;
struct N;
fn main() {
    let c = Cell::new(N);
    let f: fn(&Cell<N>) -> N = Cell::take;
    let _n = f(&c);
    println!("x");
}
