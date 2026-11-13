// panic: range end index 9 out of range for slice of length 3
fn main() {
    let mut s = String::from("abc");
    println!("before");
    let _ = s.drain(0..9);
    println!("after");
}
