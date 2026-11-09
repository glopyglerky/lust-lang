// A lifetime alias over a generic Adt, and an alias of an alias.
struct Visitor<'a, C> { name: &'a str, ctx: C }
type V<'a> = Visitor<'a, u32>;
type VR<'a> = &'a V<'a>;
fn label<'a>(v: VR<'a>) -> &'a str { v.name }
fn main() {
    let n = String::from("vis");
    let v: V<'_> = Visitor { name: &n, ctx: 3 };
    println!("{} {}", label(&v), v.ctx);
}
