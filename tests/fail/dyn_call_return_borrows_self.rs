// expect: L0306
// rustc E0597: through dyn, an elided return borrows the object, which
// dies before the result's last use.
trait Name { fn name(&self) -> &str; }
struct A { s: String }
impl Name for A { fn name(&self) -> &str { &self.s } }
fn main() {
    let r;
    {
        let a = A { s: String::from("a") };
        let d: &dyn Name = &a;
        r = d.name();
    }
    println!("{}", r);
}
