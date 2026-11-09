// expect: L0612
// rustc E0599 (`i32` is not `Deref`): core's `impl<T: Deref> Option<T>` bound fails.
fn main() {
    let x: Option<i32> = None;
    println!("{:?}", x.as_deref());
}
