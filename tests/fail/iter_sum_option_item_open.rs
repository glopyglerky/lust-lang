// expect: L0610
// rustc E0284: `u64: Sum<?U>` has two impls (`Sum<u64>`, `Sum<&u64>`), so the item type is not inferred from the sum.
fn main() {
    let p: Option<u64> = "1 2 3".split(' ').map(|w| w.parse().ok()).sum();
    println!("{:?}", p);
}
