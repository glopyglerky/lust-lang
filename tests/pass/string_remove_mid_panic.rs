// panic: start byte index 2 is not a char boundary
fn main() {
    let mut s = "héllo".to_string();
    println!("pre");
    s.remove(2);
    println!("post");
}
