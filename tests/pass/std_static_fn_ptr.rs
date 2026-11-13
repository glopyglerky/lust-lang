// A fn item as a fn pointer in static data: the static holds the
// address of a trampoline for the fn's slot.
fn four() -> i32 {
    4
}
fn twice(x: u8) -> u8 {
    x * 2
}
static F: fn() -> i32 = four;
static G: (fn(u8) -> u8, u8) = (twice, 21);
const H: Option<fn() -> i32> = Some(four);
fn main() {
    let f = F;
    let (g, n) = G;
    println!("{} {} {}", f(), g(n), match H { Some(h) => h() + 1, None => 0 });
}
