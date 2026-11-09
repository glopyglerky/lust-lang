// expect: L0101
// rustc E0603: `m`'s import of `swap` is private.
mod m {
    #[allow(unused_imports)]
    use std::mem::swap;
}
fn main() {
    let mut a = 1;
    let mut b = 2;
    m::swap(&mut a, &mut b);
}
