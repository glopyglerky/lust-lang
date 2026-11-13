// expect: L0735
// rustc E0308: two closures that capture nothing join as a fn pointer
// with their own (free) output lifetime, which is no `Fn(&u8) -> &u8`.
static G: u8 = 9;
fn ap<F: Fn(&u8) -> &u8>(f: F) -> u8 {
    let x = 3u8;
    *f(&x)
}
fn main() {
    let z = std::env::args().count();
    let f = if z > 0 { |_a: &u8| &G } else { |_a: &u8| &G };
    println!("{}", ap(f));
}
