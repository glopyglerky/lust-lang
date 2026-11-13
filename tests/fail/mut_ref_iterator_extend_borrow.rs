// expect: L0311
// rustc E0505: the items `extend(&mut a)` put into `v` borrow `w`.
fn main() {
    let w = vec![5u8, 6];
    let mut a = w.iter();
    let mut v: Vec<&u8> = Vec::new();
    v.extend(&mut a);
    drop(w);
    println!("{:?}", v);
}
