// expect: L0610
// rustc E0282: `u32: Sum<u32>` and `Sum<&u32>` both apply to an open item.
fn main() {
    let v = Vec::new();
    let s: u32 = v.into_iter().sum();
    println!("{s}");
}
