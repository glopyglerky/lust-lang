// expect: L0311
fn main() {
    let b: Box<[i32]> = vec![1, 2].into_boxed_slice();
    let r = &b[0];
    drop(b);
    println!("{}", r);
}
