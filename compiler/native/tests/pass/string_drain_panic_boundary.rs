// panic: assertion failed: self.is_char_boundary(start)
fn main() {
    let mut s = String::from("éé");
    println!("before");
    let _ = s.drain(1..2);
    println!("after");
}
