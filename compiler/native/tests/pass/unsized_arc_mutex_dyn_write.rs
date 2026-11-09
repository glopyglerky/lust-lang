use std::io::Write;
use std::sync::{Arc, Mutex};
pub type ErrorWriter = Arc<Mutex<dyn Write + Send>>;
fn report(w: &ErrorWriter, msg: &str) { let mut g = w.lock().unwrap(); writeln!(g, "error: {msg}").unwrap(); }
fn mk() -> ErrorWriter { Arc::new(Mutex::new(std::io::stdout())) }
fn main() {
    let w = mk();
    report(&w, "one");
    let w2 = w.clone();
    report(&w2, "two");
    w.lock().unwrap().flush().unwrap();
    println!("{} {}", Arc::strong_count(&w), w.is_poisoned());
}
