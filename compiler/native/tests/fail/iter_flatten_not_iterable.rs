// expect: L0612
// rustc E0277: `&i32` is not an iterator, so the items do not flatten.
fn main() {
    let v = vec![1, 2];
    let n = v.iter().flatten().count();
    println!("{n}");
}
