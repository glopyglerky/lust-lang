// expect: L0612
// rustc E0478: `trait Search: 'static` and a Self type with a lifetime.
trait Search: 'static { type Req: 'static; }
struct S<'a>(&'a u8);
impl<'a> Search for S<'a> { type Req = u8; }
fn main() {}
