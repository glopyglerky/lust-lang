// expect: L0306
// A `Box<dyn O + 'a>` drop may use `'a`: the struct holds `&'a mut` of it.
pub trait O {
    fn n(&self) -> u8;
}
struct K;
impl O for K {
    fn n(&self) -> u8 {
        1
    }
}
struct R<'a> {
    f: &'a mut Box<dyn O + 'a>,
}
fn main() {
    let mut bx: Box<dyn O + '_> = Box::new(K);
    let r = R { f: &mut bx };
    println!("{}", r.f.n());
}
