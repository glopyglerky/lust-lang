// expect: L0733
// rustc: lifetime may not live long enough. `S<'a, 'b>` implies `'b: 'a` only, not `'a: 'b`.
struct W<'w> { v: &'w u8 }
struct S<'a, 'b> { x: &'a mut W<'b> }
fn f<'a, 'b>(s: S<'a, 'b>, r: &'a u8) -> &'b u8 {
    let _ = &s;
    r
}
fn main() {}
