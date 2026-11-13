// expect: L0303
// rustc E0502: an array reaches `get_mut` through an unsize of `&mut a`,
// which is not two-phase.
fn main() {
    let mut a = [1, 2, 3];
    if let Some(x) = a.get_mut(a.len() - 1) {
        *x = 9;
    }
    println!("{:?}", a);
}
