// Dyn lifetime edits on one warm daemon: a `'static` bound on a generic
// fn, a `'static` object lifetime inside `Vec`, a `&'static dyn` field,
// and `trait T: 'static` with a reference impl. Each edit flips the
// answer, so a stale caller check shows up as a warm/cold mismatch.
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
static S0: S = S { n: 4 };
fn need<X: 'static>(_x: X) -> u32 {
    7
}
fn call(d: &dyn T) -> u32 {
    d.read()
}
fn keep(v: Vec<&dyn T>) -> Vec<&dyn T> {
    v
}
struct H {
    x: &'static dyn T,
}
fn main() {
    let s = S { n: 2 };
    let h = H { x: &S0 };
    println!("{} {} {} {}", need(S { n: 1 }), call(&s), keep(vec![&s])[0].read(), h.x.read());
}
