// A std source module (core::time) from a non-root file.
use std::time::Duration;

pub fn go() {
    let d = Duration::from_millis(1500);
    println!("{} {}", d.as_secs(), d.subsec_millis());
}
