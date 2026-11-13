// `&mut *b` reborrows a `Box<dyn Tr>` as `&mut dyn Tr`; `&mut self`
// methods through the box.
trait Counter { fn bump(&mut self) -> u32; fn get(&self) -> u32; }
struct C(u32);
impl Counter for C { fn bump(&mut self) -> u32 { self.0 += 1; self.0 } fn get(&self) -> u32 { self.0 } }
fn run(c: &mut dyn Counter) { c.bump(); }
fn main() {
    let mut b: Box<dyn Counter> = Box::new(C(5));
    b.bump();
    run(&mut *b);
    println!("{}", b.get());
}
