// expect: L0735
// rustc: implementation of `FnMut` is not general enough. The joined
// fn pointer returns its own lifetime, not its argument's.
static G: u8 = 9;
fn mk(z: u8) -> impl FnMut(usize, &u8) -> &u8 {
    if z > 0 { move |_: usize, _a: &u8| &G } else { move |_: usize, _a: &u8| &G }
}
fn main() {
    let mut f = mk(1);
    println!("{}", f(0, &2));
}
