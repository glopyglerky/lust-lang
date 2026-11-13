use std::sync::Arc;
fn main() { let a: Arc<str> = Arc::from("hi"); println!("[{:>6}] [{:<4}] [{:^6}] [{:?}] {}", a, a, a, a, a.to_string()); let s: Arc<[u16]> = Arc::from(vec![1, 2]); println!("{:?} {:#?}", s, s); }
