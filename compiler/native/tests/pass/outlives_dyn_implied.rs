// Dyn coercions of a type param that its fn's implied bounds or a
// `'static` bound allow.
trait Obj {
    fn v(&self) -> usize;
}
impl Obj for u8 {
    fn v(&self) -> usize {
        *self as usize
    }
}
impl<X> Obj for Vec<X> {
    fn v(&self) -> usize {
        self.len()
    }
}
fn by_arg<'a, T: Obj>(x: T, _y: &'a T) -> Box<dyn Obj + 'a> {
    Box::new(x)
}
fn by_vec<'a, T>(x: Vec<T>, _y: &'a Vec<T>) -> Box<dyn Obj + 'a> {
    Box::new(x)
}
fn local_only<T: Obj>(x: T) -> usize {
    let b: Box<dyn Obj + '_> = Box::new(x);
    b.v()
}
fn pushed<'a, T: Obj + 'static>(x: T, out: &mut Vec<Box<dyn Obj + 'a>>) {
    out.push(Box::new(x));
}
fn main() {
    let (a, b) = (3u8, vec![1u8, 2]);
    println!("{}", by_arg(5u8, &a).v());
    println!("{}", by_vec(vec![1u8], &b).v());
    println!("{}", local_only(9u8));
    let mut out: Vec<Box<dyn Obj>> = Vec::new();
    pushed(2u8, &mut out);
    println!("{}", out[0].v());
}
