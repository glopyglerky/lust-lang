// expect: L0306
// rustc E0597: the impl returns its own `'a` where the trait borrows
// `self`; callers see the trait's signature, so `s` must outlive `r`.
trait Tr { fn get(&self) -> &String; }
struct S<'a> { r: &'a String }
impl<'a> Tr for S<'a> { fn get(&self) -> &'a String { self.r } }
fn main() {
    let x = String::from("x");
    let r;
    {
        let s = S { r: &x };
        r = s.get();
    }
    println!("{r}");
}
