// expect: L0112
// rustc E0369: RecvTimeoutError is PartialEq and Eq, not PartialOrd.
use std::sync::mpsc::RecvTimeoutError;
fn main() { println!("{}", RecvTimeoutError::Timeout < RecvTimeoutError::Disconnected); }
