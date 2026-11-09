use std::sync::Arc;
fn main() { let k = 3; let f = Arc::new(move |x: i32| x * k); let g = f.clone(); println!("{} {}", f(2), g(5)); }
