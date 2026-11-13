// expect: L0104
// rustc E0277: no `TryFrom<&Vec<u8>> for [u8; 2]` (the blanket TryInto is picked at the `&Vec` step).
fn main() {
    let v = vec![1u8, 2];
    let a: [u8; 2] = (&v).try_into().unwrap();
    println!("{:?}", a);
}
