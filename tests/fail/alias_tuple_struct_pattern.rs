// expect: L0102
// rustc E0532: a tuple-struct pattern cannot go through a type alias.
struct P(u8, u16);
type PA = P;
fn main() {
    let p = P(1, 2);
    let PA(x, y) = p;
    println!("{} {}", x, y);
}
