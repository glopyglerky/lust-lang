// panic: range end index 5 out of range for slice of length 3
fn main() {
    let mut v = vec![1, 2, 3];
    println!("before");
    let _ = v.drain(0..5);
    println!("after");
}
