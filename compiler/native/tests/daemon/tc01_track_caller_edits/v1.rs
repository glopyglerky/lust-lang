// #[track_caller] on one warm daemon: the attribute added to a fn whose
// callers do not change (its C signature gains the location), a caller
// that becomes #[track_caller] itself (the location propagates), and the
// attribute removed again.
use std::panic::Location;
fn where_am_i() -> u32 {
    Location::caller().line()
}
fn middle() -> u32 {
    where_am_i()
}
fn main() {
    println!("{} {}", where_am_i(), middle());
    let v: Vec<u8> = vec![1];
    let i = v.len();
    println!("{}", at(&v, i - 1));
}
fn at(v: &[u8], i: usize) -> u8 {
    v[i]
}
