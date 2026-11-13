// expect: L0306
// rustc E0597: `a` borrows from `v` through `v.iter()`, and `v` ends before `keep` is used.
fn main() {
    let mut keep = Vec::new();
    {
        let v = vec![(1u8, 2u8)];
        for (a, _b) in v.iter() {
            keep.push(a);
        }
    }
    println!("{:?}", keep);
}
