// A supertrait method through `&dyn B` (upcast to `dyn A`) takes the
// supertrait's signature: its `'static` result outlives the object.
trait A { fn a(&self) -> &'static str; }
trait B: A { fn b(&self) -> u32; }
struct S { n: u32 }
impl A for S { fn a(&self) -> &'static str { "a" } }
impl B for S { fn b(&self) -> u32 { self.n } }
fn main() {
    let r;
    {
        let s = S { n: 3 };
        let d: &dyn B = &s;
        r = d.a();
        println!("{}", d.b());
    }
    println!("{}", r);
}
