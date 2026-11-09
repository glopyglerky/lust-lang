// String methods assert char boundaries with rustc's text.
// panic: assertion failed: self.is_char_boundary(idx)
fn main() {
    let n = std::env::args().count();
    let mut s = String::from("héllo");
    s.insert(n - 1, '>');
    println!("{}", s);
    println!("{}", s.remove(n));
    s.truncate(n + 3);
    println!("{}", s);
    s.insert_str(n + 1, "x");
}
