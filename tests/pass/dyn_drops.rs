// ARCH 2.8: drops of concrete values behind `&dyn` happen at their own scope ends.
trait Tr { fn hi(&self) -> u8; }
struct Loud(u8);
impl Drop for Loud { fn drop(&mut self) { println!("drop {}", self.0); } }
impl Tr for Loud { fn hi(&self) -> u8 { self.0 } }
fn use_it(t: &dyn Tr) -> u8 { t.hi() }
fn main() {
    let a = Loud(1);
    {
        let b = Loud(2);
        let d: &dyn Tr = &b;
        println!("{}", use_it(d) + use_it(&a));
    }
    let t: &dyn Tr = &Loud(3);
    println!("{}", t.hi());
    println!("end");
}
