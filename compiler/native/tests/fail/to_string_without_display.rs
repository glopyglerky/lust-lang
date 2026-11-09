// expect: L0104
// rustc E0599: a struct with Debug but no Display has no `to_string`.
#[derive(Debug)]
struct D(u8);
fn main() {
    let s = D(7).to_string();
    println!("{}", s);
}
