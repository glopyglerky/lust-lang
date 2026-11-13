// A blanket impl whose where clause fails for the receiver is no
// method candidate: `Box<dyn V>` and `&dyn V` reach the object method.
trait Tag {
    fn tag(&self) -> u32;
}
trait V {
    fn name(&self) -> u32;
}
impl<T: Tag> V for T {
    fn name(&self) -> u32 {
        self.tag() + 100
    }
}
struct A;
impl Tag for A {
    fn tag(&self) -> u32 {
        1
    }
}
struct B(u32);
impl V for B {
    fn name(&self) -> u32 {
        self.0
    }
}
fn main() {
    let vs: Vec<Box<dyn V>> = vec![Box::new(A), Box::new(B(7))];
    for v in &vs {
        println!("{}", v.name());
    }
    let r: &dyn V = &B(9);
    println!("{}", r.name());
    println!("{}", A.name());
    println!("{}", (&A).name());
}
