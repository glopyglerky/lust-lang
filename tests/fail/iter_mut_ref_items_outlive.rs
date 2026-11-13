// expect: L0303
// rustc E0502: the items collected through `&mut it` borrow `v`, not `it`.
fn main() {
    let mut v = vec![1u32, 2, 3];
    let mut it = v.iter();
    let xs: Vec<&u32> = (&mut it).collect();
    v.push(9);
    println!("{:?}", xs);
}
