// expect: L0306
// rustc E0597: the result borrows through the object; the data the
// object borrows dies first.
trait Tr { fn get(&self) -> &String; }
struct S<'a> { r: &'a String }
impl<'a> Tr for S<'a> { fn get(&self) -> &String { self.r } }
fn main() {
    let r;
    {
        let x = String::from("x");
        let s = S { r: &x };
        let d: &dyn Tr = &s;
        r = d.get();
    }
    println!("{}", r);
}
