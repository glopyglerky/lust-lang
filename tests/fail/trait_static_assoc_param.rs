// expect: L0612
// rustc E0310: the binding `Req = T` needs `T: 'static`.
trait Search { type Req: 'static; }
struct W<T>(T);
impl<T> Search for W<T> { type Req = T; }
fn main() {}
