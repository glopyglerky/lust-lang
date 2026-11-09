// expect: L0101
// rustc E0423: a type alias names no constructor in the value namespace.
struct P(u8, u16);
type PA = P;
fn main() {
    let p = PA(1, 2);
    println!("{}", p.0);
}
