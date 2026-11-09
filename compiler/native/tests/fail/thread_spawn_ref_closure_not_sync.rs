// expect: L0612
// rustc E0277: `&F: Send` needs `F: Sync`; a Cell capture is not Sync.
fn main() { let c = std::cell::Cell::new(0); let w = || { c.set(1); }; std::thread::scope(|s| { s.spawn(&w); }); }
