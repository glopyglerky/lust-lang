use std::sync::Arc;
fn main() {
    let a: Arc<str> = Arc::from(String::from("world"));
    let b = Arc::clone(&a);
    println!("{a} {} {}", b.to_uppercase(), Arc::strong_count(&a));
    let c: Arc<str> = "x".into();
    println!("{} {}", a == b, a == c);
    println!("{:?}", a.cmp(&c));
}
