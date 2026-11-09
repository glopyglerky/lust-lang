// expect: L0305
// rustc E0507: a field through a user `Deref` cannot move out.
use std::ops::Deref;
struct Late { name: String }
struct Info { late: Late }
impl Deref for Info { type Target = Late; fn deref(&self) -> &Late { &self.late } }
fn main() { let info = Info { late: Late { name: String::from("x") } }; let n = info.name; println!("{}", n); }
