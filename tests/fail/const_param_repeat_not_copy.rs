// expect: L0110
// rustc E0277: a repeat with a const param length needs a Copy element.
fn s<const N: usize>() -> Vec<String> {
    let a = [String::new(); N];
    a.to_vec()
}
fn main() {
    println!("{:?}", s::<2>());
}
