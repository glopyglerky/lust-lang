// Dropping `Box<dyn Tr>` runs the concrete type's drop through the
// vtable, then frees the box: `drop`, a Vec pop, and scope end.
trait Speak { fn hi(&self) -> String; }
struct D(&'static str);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
impl Speak for D { fn hi(&self) -> String { format!("hi {}", self.0) } }
struct Pair { a: D, b: D }
impl Speak for Pair { fn hi(&self) -> String { format!("{}+{}", self.a.0, self.b.0) } }
fn make(n: u8) -> Box<dyn Speak> { if n == 0 { Box::new(D("zero")) } else { Box::new(Pair { a: D("pa"), b: D("pb") }) } }
fn main() {
    let x = make(0);
    let y = make(1);
    println!("{} {}", x.hi(), y.hi());
    drop(x);
    println!("after x");
    let mut v: Vec<Box<dyn Speak>> = Vec::new();
    v.push(make(0));
    v.push(Box::new(D("pushed")));
    println!("{}", v.len());
    v.pop();
    println!("end");
}
