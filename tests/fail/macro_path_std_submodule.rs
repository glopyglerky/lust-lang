// expect: L0802
// rustc E0433: std::vec is the module of `Vec`; the `vec!` macro lives at the std root.
fn main() {
    let v: Vec<i32> = std::vec::vec![1];
    println!("{v:?}");
}
