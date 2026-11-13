// expect: L0612
// rustc E0277: `u8` does not implement `Borrow<[u8]>`.
use std::borrow::Borrow;

fn f<B: Borrow<[u8]>>(b: B) -> usize {
    b.borrow().len()
}

fn main() {
    println!("{}", f(3u8));
}
