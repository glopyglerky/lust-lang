// A lifetime alias in a struct field and in a trait method's return.
type R<'a> = &'a String;
struct W<'a> { r: R<'a> }
trait Name { fn name(&self) -> R<'_>; }
impl<'a> Name for W<'a> { fn name(&self) -> R<'_> { self.r } }
fn main() { let s = String::from("w"); let w = W { r: &s }; println!("{}", w.name()); }
