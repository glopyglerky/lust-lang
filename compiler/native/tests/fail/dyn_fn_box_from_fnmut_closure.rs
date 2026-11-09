// expect: L0612
// rustc E0594: a closure that mutates a capture is not `dyn Fn`.
fn main() { let mut n = 0; let f: Box<dyn Fn()> = Box::new(|| n += 1); f(); }
