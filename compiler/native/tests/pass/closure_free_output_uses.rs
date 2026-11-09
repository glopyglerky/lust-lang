// Closures with their own (free) output lifetime, used where rustc
// accepts them: called directly, joined and called with a `'static`
// result, joined into an elided fn pointer, and passed where written.
static G: u8 = 9;
fn ap<F: Fn(&u8) -> &u8>(f: F) -> u8 {
    let x = 3u8;
    *f(&x)
}
fn apt<T, F: Fn(&u8) -> T>(f: F) -> T {
    let x = 3u8;
    f(&x)
}
fn main() {
    let z = std::env::args().count();
    let f = |_a: &u8| &G;
    let k = 1u8;
    let r1 = f(&k);
    let g = if z > 0 { |_a: &u8| &G } else { |_a: &u8| &G };
    let r2: &'static u8 = { let x = 4u8; g(&x) };
    let h: fn(&u8) -> &u8 = g;
    let r3 = ap(h) + ap(|_a: &u8| &G) + *apt(f);
    let k2 = if z > 0 { |a: &u8| *a } else { |a: &u8| *a + 1 };
    println!("{} {} {} {}", r1, r2, r3, ap(|a| a) + k2(&5));
}
