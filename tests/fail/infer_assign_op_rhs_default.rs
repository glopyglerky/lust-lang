// expect: L0610
// rustc E0283: `u8: AddAssign<?R>` has `Rhs = u8` and `Rhs = &u8`, so
// `x += Default::default()` leaves the right side's type open.
fn main() {
    let mut x = 3u8;
    x += Default::default();
    println!("{}", x);
}
