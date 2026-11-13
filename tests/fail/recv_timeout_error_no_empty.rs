// expect: L0104
// rustc E0599: RecvTimeoutError has Timeout and Disconnected, no Empty.
use std::sync::mpsc::RecvTimeoutError;
fn main() { let _e = RecvTimeoutError::Empty; }
