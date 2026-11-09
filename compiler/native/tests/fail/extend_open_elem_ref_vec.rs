// expect: L0610
// rustc E0282: Vec<T> is Extend<T> and Extend<&T>; a later len() fixes nothing.
fn main() {
    let src = vec![1u8, 2];
    let mut v = Vec::new();
    v.extend(src.iter());
    println!("{}", v.len());
}
