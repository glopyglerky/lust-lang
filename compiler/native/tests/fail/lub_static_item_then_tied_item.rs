// expect: L0110
// rustc E0308: the join keeps the first fn's signature (`'static`
// output), and `tied` returns its argument's lifetime.
static G: u8 = 9;
fn tied(a: &u8) -> &u8 {
    a
}
fn stat(_a: &u8) -> &'static u8 {
    &G
}
fn main() {
    let z = std::env::args().count();
    let f = if z > 0 { stat } else { tied };
    println!("{}", f(&1));
}
