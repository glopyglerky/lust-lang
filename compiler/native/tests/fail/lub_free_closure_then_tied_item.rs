// expect: L0110
// rustc E0308: the closure's own output lifetime comes first; `tied`
// is no subtype of it.
static G: u8 = 9;
fn tied(a: &u8) -> &u8 {
    a
}
fn main() {
    let z = std::env::args().count();
    let f = if z > 0 { |_a: &u8| &G } else { tied };
    println!("{}", f(&1));
}
