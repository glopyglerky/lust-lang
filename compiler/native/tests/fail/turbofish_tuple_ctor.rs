// expect: L0110
// rustc E0308: the turbofish fixes `T = u8`; a `&str` field is a mismatch.
struct P<T>(T);

fn main() {
    let p = P::<u8>("x");
    println!("{}", p.0);
}
