use std::sync::Mutex;
fn f<'a>(m: &'a Mutex<&'a i32>) -> &'a Mutex<&'a i32> { m }
fn main() { let a = 1; let b = 2; let m0 = Mutex::new(&a); let m = f(&m0); *m.lock().unwrap() = &b; println!("{}", m.lock().unwrap()); }
