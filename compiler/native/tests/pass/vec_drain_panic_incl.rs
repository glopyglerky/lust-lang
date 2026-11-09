// panic: range end index 3 out of range for slice of length 3
fn main() {
    let mut v = vec![1, 2, 3];
    println!("before");
    let _ = v.drain(..=3);
    println!("after");
}
