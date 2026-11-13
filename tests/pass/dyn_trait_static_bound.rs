// ARCH 2.8: a trait bounded by `'static` (directly, by `where Self: 'static`, or through a supertrait)
// makes every `dyn` of it `'static`, as in rustc, so uses that need `'static` take a `&dyn` param.
trait A: 'static { fn a(&self) -> u8; fn h(&self) -> u8 where Self: 'static { 1 } }
trait W where Self: 'static { fn w(&self) -> u8; }
trait B { fn b(&self) -> u8; }
trait Sub: A + B { fn s(&self) -> u8; }
trait O { fn o(&self) -> u8; }
impl dyn A { fn ia(&self) -> u8 { 10 + self.a() } }
impl dyn W { fn iw(&self) -> u8 { 20 + self.w() } }
impl dyn Sub { fn is(&self) -> u8 { 30 + self.s() + self.b() } }
impl O for dyn A { fn o(&self) -> u8 { 40 + self.a() } }
struct S(u8);
impl A for S { fn a(&self) -> u8 { self.0 } }
impl W for S { fn w(&self) -> u8 { self.0 + 1 } }
impl B for S { fn b(&self) -> u8 { self.0 + 2 } }
impl Sub for S { fn s(&self) -> u8 { self.0 + 3 } }
fn ca(d: &dyn A) -> u8 { d.ia() + d.o() + d.h() }
fn cw(d: &dyn W) -> u8 { d.iw() }
fn cs(d: &dyn Sub) -> u8 { d.is() + d.h() }
fn up(d: &dyn Sub) -> &(dyn A + 'static) { d }
fn up_b(d: &dyn Sub) -> &(dyn B + 'static) { d }
fn main() {
    let s = S(3);
    println!("{} {} {}", ca(&s), cw(&s), cs(&s));
    println!("{} {}", up(&s).a(), up_b(&s).b());
}
