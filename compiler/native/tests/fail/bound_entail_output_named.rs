// expect: L0733
// rustc: lifetime may not live long enough. `F: Fn() -> &'a u8` passed for `G: Fn() -> &'b u8` makes `'b` be `'a`, so the result is not `'static`.
fn inner<'b, G: Fn() -> &'b u8>(g: G) -> &'b u8 {
    g()
}

fn outer<'a, F: Fn() -> &'a u8>(f: F) -> &'static u8 {
    inner(f)
}

fn main() {}
