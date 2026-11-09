// expect: L0610
// rustc E0282: `7 + *x` has `i32: Add<i32>` and `Add<&i32>`, so the open
// right operand picks neither.
fn main() {
    let v = Vec::new();
    let b: Vec<i32> = v.iter().map(|x| 7 + *x).collect();
    println!("{b:?} {}", v.len());
}
