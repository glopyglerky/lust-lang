// Coherence and orphan shapes rustc accepts: a non-fundamental type covers T
// (RFC 2451), Fn is fundamental (knowable for negative reasoning), and a local
// type with no impl cannot gain one upstream.
struct Local;
impl<T> From<Local> for Vec<T> { fn from(_: Local) -> Self { Vec::new() } }
trait Tr { fn f(&self) -> i32; }
impl<T: Fn() -> i32> Tr for T { fn f(&self) -> i32 { self() } }
impl Tr for i32 { fn f(&self) -> i32 { 2 } }
impl Tr for Local { fn f(&self) -> i32 { 3 } }
trait Tag { fn tag(&self) -> &'static str; }
impl<T: std::error::Error> Tag for T { fn tag(&self) -> &'static str { "error" } }
impl Tag for Local { fn tag(&self) -> &'static str { "local" } }
struct NoHash;
impl std::hash::Hash for &NoHash { fn hash<H: std::hash::Hasher>(&self, _: &mut H) {} }
fn seven() -> i32 { 7 }
fn main() {
    let v: Vec<u8> = Local.into();
    println!("{} {} {} {}", v.len(), seven(), 5i32.f(), Local.f());
    println!("{}", Local.tag());
}
