// feature: threads.send_sync_bounds
// form: dyn Trait + Send, as in ts-rust api/server dyn Read + Send fields
trait Greeter { fn hi(&self) -> i32; }
struct A;
impl Greeter for A { fn hi(&self) -> i32 { 1 } }
fn call(g: &(dyn Greeter + Send + Sync)) -> i32 { g.hi() }
fn main() { let a = A; println!("{}", call(&a)); }
