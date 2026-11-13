// expect: L0301
// rustc E0382: a builtin iterator is Clone but not Copy, so a move leaves
// the original unusable after its clone.
fn main() {
    let v = vec![1u8, 2];
    let a = v.iter();
    let b = a.clone();
    let c = a;
    println!("{} {} {}", a.count(), b.count(), c.count());
}
