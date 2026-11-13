// expect: L0612
// rustc E0277: `String: From<u32>` does not hold.
fn main() {
    let v: Vec<String> = vec![1u32].into_iter().map(String::from).collect();
    println!("{v:?}");
}
