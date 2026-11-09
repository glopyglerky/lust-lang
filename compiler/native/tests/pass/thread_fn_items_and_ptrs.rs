use std::sync::{OnceLock, Mutex, Condvar};
use std::thread;
fn w() -> u32 { 5 }
fn big() -> [u64; 8] { [3; 8] }
fn init() -> String { String::from("init") }
fn not_zero(v: &mut u32) -> bool { *v == 0 }
fn sc<'s, 'e>(s: &'s thread::Scope<'s, 'e>) -> u8 { s.spawn(|| 2u8).join().unwrap() }
static O: OnceLock<String> = OnceLock::new();
fn main() {
    println!("{}", thread::spawn(w).join().unwrap());
    println!("{:?}", thread::spawn(big).join().unwrap());
    let p: fn() -> u32 = w;
    println!("{}", thread::spawn(p).join().unwrap());
    println!("{}", O.get_or_init(init));
    let m = Mutex::new(1u32); let c = Condvar::new();
    let g = c.wait_while(m.lock().unwrap(), not_zero).unwrap();
    println!("{} {}", *g, thread::scope(sc));
}
