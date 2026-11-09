// panic: assertion failed: self.is_char_boundary(end)
fn main() {
    let mut s = String::from("éé");
    println!("before");
    let _ = s.drain(0..1);
    println!("after");
}
