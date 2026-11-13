// S::clone(&self) applies at the &S step's by-value probe, before core's
// Clone for &S (which needs &&S), so rustc calls the inherent method.
#[derive(Debug)]
struct S(u32);
impl S {
    fn clone(&self) -> u32 {
        970 + self.0
    }
}
fn main() {
    let v = S(2);
    let r = &v;
    println!("{:?} {:?}", r.clone(), v.clone());
}
