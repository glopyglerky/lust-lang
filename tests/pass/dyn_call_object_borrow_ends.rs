// The object borrows `x`, but a `'static` result through dyn does not;
// `x` can move after the object is gone.
trait Tr { fn tag(&self) -> &'static str; }
struct S<'a> { r: &'a String }
impl Tr for S<'_> { fn tag(&self) -> &'static str { if self.r.len() > 1 { "long" } else { "short" } } }
fn main() {
    let x = String::from("xy");
    let r;
    {
        let s = S { r: &x };
        let d: &dyn Tr = &s;
        r = d.tag();
    }
    drop(x);
    println!("{}", r);
}
