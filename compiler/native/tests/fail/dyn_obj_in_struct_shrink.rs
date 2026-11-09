// expect: L0733
// rustc: lifetime may not live long enough. `H<'a>` holds `&'a (dyn T +
// 'a)`; it does not become `H<'b>` (fx-dyn-review).
trait T { fn r(&self) -> u32; } impl T for u32 { fn r(&self) -> u32 { *self } }
struct H<'a> { d: &'a (dyn T + 'a) } fn f<'a, 'b>(h: H<'a>) -> H<'b> { h } fn main() { let n = 1u32; let h = H { d: &n }; println!("{}", f(h).d.r()); }
