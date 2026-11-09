// expect: L0112
// rustc E0204: `impl<T: Copy> Copy for Outer<T>` needs every field to be
// Copy, and `Vec<T>` is not.
#[derive(Clone, Copy)]
struct Outer<T> {
    v: Vec<T>,
}
fn main() {
    let o: Outer<u8> = Outer { v: vec![1] };
    println!("{}", o.v.len());
}
