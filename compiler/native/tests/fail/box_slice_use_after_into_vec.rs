// expect: L0301
fn main() {
    let b: Box<[i32]> = vec![1, 2].into_boxed_slice();
    let v = b.into_vec();
    println!("{:?} {:?}", v, b);
}
