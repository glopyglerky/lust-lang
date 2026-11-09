// expect: L0306
// rustc E0597: `w` is dropped while the reborrowed subslice is still used.
fn main() {
    let r: &[u8];
    {
        let w = vec![1u8, 2, 3, 4];
        r = &w[1..][..2];
    }
    println!("{:?}", r);
}
