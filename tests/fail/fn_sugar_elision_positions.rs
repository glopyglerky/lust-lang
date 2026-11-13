// expect: L0202
// rustc E0106: in fn pointer types, `dyn Fn` and `impl Fn` sugar, and bounds, an elided output lifetime needs exactly one input lifetime.
fn a(f: fn() -> &u8) {}
fn b(f: &dyn Fn(&u8, &u8) -> &u8) {}
fn c(f: impl Fn() -> &u8) {}
struct S<F>(F);
impl<F: Fn(&u8, &u8) -> &u8> S<F> {}

fn main() {}
