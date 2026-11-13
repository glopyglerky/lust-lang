// expect: L0110
// rustc E0161: a tuple element moves its value; `[u32]` has no known size.
fn main() {
    let b: Box<[u32]> = vec![1u32, 2].into_boxed_slice();
    let x = (*b, 1).1;
    println!("{}", x);
}
