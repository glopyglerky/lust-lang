// expect: L0612
// rustc E0525: `&F` is FnOnce only when F is Fn; this closure mutates a capture.
fn main() { let mut c = 0; let mut w = || { c += 1; }; std::thread::scope(|s| { s.spawn(&w); }); }
