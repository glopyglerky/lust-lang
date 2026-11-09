// expect: L0303
// rustc E0502: `get_mut` on a `Vec` goes through `DerefMut` (not two-phase).
fn main() {
    let mut v = vec![1, 2, 3];
    if let Some(x) = v.get_mut(v.len() - 1) {
        *x = 9;
    }
    println!("{:?}", v);
}
