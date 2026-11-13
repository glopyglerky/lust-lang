// expect: L0110
// rustc E0277: a closure's return value must have a known size.
fn main() {
    let v = vec![1u32, 2, 3];
    let n = (|| v[1..])().len();
    println!("{}", n);
}
