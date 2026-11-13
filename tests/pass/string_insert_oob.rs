// panic: assertion failed: self.is_char_boundary(idx)
fn main() {
    let mut s = "héllo".to_string();
    println!("pre");
    s.insert(9, 'x');
    println!("post");
}
