// `type Out = dyn Tr` is `dyn Tr + 'static`, so `&(dyn Tr + 'static)` is
// the trait's `&Self::Out` (W-STD-B, object lifetime defaults).
trait Tr { fn hi(&self) -> u8; }
struct A;
impl Tr for A { fn hi(&self) -> u8 { 1 } }
trait Get { type Out: ?Sized; fn get(&self) -> &Self::Out; }
struct H { b: Box<dyn Tr> }
impl Get for H { type Out = dyn Tr; fn get(&self) -> &(dyn Tr + 'static) { &*self.b } }
fn main() { let h = H { b: Box::new(A) }; println!("{}", h.get().hi()); }
