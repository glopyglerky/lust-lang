// ARCH 2.8: a diamond of supertraits: calls, overrides, and upcasts along two paths.
trait Base { fn id(&self) -> u32; fn base_name(&self) -> String { format!("base{}", self.id()) } }
trait Left: Base { fn left(&self) -> u32 { self.id() + 10 } }
trait Right: Base { fn right(&self) -> u32 { self.id() + 20 } }
trait Both: Left + Right { fn both(&self) -> u32 { self.left() + self.right() } }
struct S(u32);
impl Base for S { fn id(&self) -> u32 { self.0 } }
impl Left for S {}
impl Right for S { fn right(&self) -> u32 { 99 } }
impl Both for S {}
fn via_base(b: &dyn Base) -> String { b.base_name() }
fn via_left(l: &dyn Left) -> u32 { l.left() + l.id() }
fn via_right(r: &dyn Right) -> u32 { r.right() }
fn main() {
    let s = S(1);
    let b: &dyn Both = &s;
    println!("{} {} {} {}", b.both(), b.left(), b.right(), b.id());
    let l: &dyn Left = b;
    let r: &dyn Right = b;
    let base: &dyn Base = b;
    let base2: &dyn Base = l;
    println!("{} {} {} {} {}", via_left(l), via_right(r), via_base(base), via_base(base2), via_base(r));
    println!("{}", b.base_name());
}
