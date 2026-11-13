// The hook prints a `String` payload's text, not `Box<dyn Any>`.
fn main() {
    println!("start");
    let s = format!("n={}", 3);
    std::panic::panic_any(s);
}
