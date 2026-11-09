// expect: L0733
// rustc E0309: a `Vec<T>` pushed as `Box<dyn Obj + 'a>` needs `T: 'a`.
trait Obj { fn v(&self) -> usize; }
impl<X> Obj for Vec<X> { fn v(&self) -> usize { self.len() } }
fn f<'a, T>(x: Vec<T>, out: &mut Vec<Box<dyn Obj + 'a>>) { out.push(Box::new(x)); }
fn main() {
    let mut out: Vec<Box<dyn Obj>> = Vec::new();
    f(vec![1u8], &mut out);
    println!("{}", out[0].v());
}
