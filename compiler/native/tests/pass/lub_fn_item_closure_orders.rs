// Joins of fn items and closures that rustc accepts: the first side's
// signature is kept and the later sides are subtypes of it.
static G: u8 = 9;
fn tied(a: &u8) -> &u8 {
    a
}
fn stat(_a: &u8) -> &'static u8 {
    &G
}
fn ap<F: Fn(&u8) -> &u8>(f: F) -> u8 {
    *f(&3)
}
fn main() {
    let z = std::env::args().count();
    let a = if z > 0 { tied } else { |_a: &u8| &G };
    let b = if z > 0 { stat } else { |_a: &u8| &G };
    let c = if z > 0 { |_a: &u8| &G } else { stat };
    let d = if z > 0 { tied } else { stat };
    let e = match z { 0 => tied, 1 => |_a: &u8| &G, _ => |_a: &u8| &G };
    let r: &'static u8 = { let x = 4u8; c(&x) };
    let g: fn(&u8) -> &u8 = b;
    println!("{} {} {} {} {} {}", ap(a), ap(g), r, ap(d), ap(e), [tied, |_a: &u8| &G][0](&7));
}
