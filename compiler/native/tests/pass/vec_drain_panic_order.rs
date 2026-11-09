// panic: slice index starts at 4 but ends at 2
fn main() {
    let mut v = vec![1, 2, 3, 4, 5];
    println!("before");
    let _ = v.drain(4..2);
    println!("after");
}
