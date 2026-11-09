// panic: unwrap_panic_column.rs:5:30
// A track_caller method call reports the method name position.
fn main() {
    let v = vec![1, 2, 3];
    println!("{}", *v.get(5).unwrap());
}
