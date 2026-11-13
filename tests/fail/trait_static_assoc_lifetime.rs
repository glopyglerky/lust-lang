// expect: L0612
// rustc E0478: the binding of `Req: 'static` holds the impl's `'a`.
trait Search { type Req: 'static; }
struct S<'a>(&'a u8);
impl<'a> Search for S<'a> { type Req = S<'a>; }
fn main() {}
