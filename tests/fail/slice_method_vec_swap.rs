// expect: L0303
// rustc E0502: `swap` is a slice method. `v` reaches the slice through
// `DerefMut`, whose `&mut v` is not two-phase.
fn main() {
    let mut v = vec![1, 2, 3];
    v.swap(0, v.len() - 1);
    println!("{:?}", v);
}
