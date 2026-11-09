// expect: L0116
// rustc E0604: only `u8` casts to `char`; the operand is `f32` once the
// blanket impl fixes the closure parameter.
trait Sink {
    fn child(&mut self, x: f32) -> u64;
}
impl<F: FnMut(f32) -> u64> Sink for F {
    fn child(&mut self, x: f32) -> u64 {
        self(x)
    }
}
fn run(mut s: impl Sink) -> u64 {
    s.child(2.5)
}
fn main() {
    println!("{}", run(|c| {
        let y = c as char;
        y as u64
    }));
}
