// ARCH 2.8: `Vec<&dyn>`, `Option<&dyn>`, tuples of `&dyn`.
trait Tr { fn v(&self) -> u32; }
struct A(u32);
impl Tr for A { fn v(&self) -> u32 { self.0 } }
fn first(xs: &Vec<&dyn Tr>) -> Option<u32> { match xs.first() { Some(d) => Some(d.v()), None => None } }
fn main() {
    let a = A(3);
    let b = A(4);
    let mut xs: Vec<&dyn Tr> = Vec::new();
    println!("{:?}", first(&xs));
    xs.push(&a);
    xs.push(&b);
    println!("{:?} {}", first(&xs), xs.len());
    let o: Option<&dyn Tr> = if xs.len() > 1 { Some(xs[1]) } else { None };
    if let Some(d) = o { println!("{}", d.v()); }
    let t: (&dyn Tr, u32) = (xs[0], 9);
    println!("{} {}", t.0.v(), t.1);
}
