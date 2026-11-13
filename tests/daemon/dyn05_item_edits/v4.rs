// Item edits that flip a dyn answer while the body that depends on
// them (`g`) stays the same text: a `'static` bound on a callee, a
// `'static` object lifetime on a callee parameter, and the header of an
// `impl dyn T` block. A warm daemon may reuse `g`'s stored check only
// while those items are unchanged.
trait T {
    fn read(&self) -> u32;
}
struct S {
    n: u32,
}
impl T for S {
    fn read(&self) -> u32 {
        self.n
    }
}
impl dyn T {
    fn more(&self) -> u32 {
        self.read() + 1
    }
}
fn need<X>(_x: X) -> u32 {
    7
}
fn call(d: &dyn T) -> u32 {
    d.read()
}
fn g(e: &dyn T) -> u32 {
    need(e) + call(e) + e.more()
}
fn main() {
    let s = S { n: 2 };
    println!("{}", g(&s));
}
