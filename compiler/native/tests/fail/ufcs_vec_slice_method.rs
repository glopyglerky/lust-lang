// expect: L0104
// rustc E0599: `iter` is a slice method; `Vec::iter` names no item.
fn main() {
    let v = vec![1, 2];
    println!("{}", Vec::iter(&v).count());
}
