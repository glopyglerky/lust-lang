// stage: W-THREADS
// ARCH 2.13: the real ts-rust shapes at core.rs:472,
// `[const { OnceLock::new() }; CHUNKS]`, plus a Mutex::new
// static. Both gate with W-THREADS in wave C. The A1 form is
// tests/consts/pass/local_once_repeat.rs.
use std::sync::{Mutex, OnceLock};
const CHUNKS: usize = 4;
static TEXTS: [OnceLock<&'static str>; CHUNKS] = [const { OnceLock::new() }; CHUNKS];
static COUNT: Mutex<u32> = Mutex::new(10);
fn main() {
    let set = TEXTS[1].set("named");
    let mut count = COUNT.lock().unwrap();
    *count += 1;
    println!("{} {} {}", set.is_ok(), TEXTS[1].get().unwrap(), count);
}
