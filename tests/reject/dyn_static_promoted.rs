// expect: L0705
// Valid Rust: rustc promotes `&S { n: 5 }` to a 'static constant. lust proves `&'static` only for a borrow of a static item.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
fn main() { let d: &'static dyn T = &S { n: 5 }; println!("{}", d.read()); }
