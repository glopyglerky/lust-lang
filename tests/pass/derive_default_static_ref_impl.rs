// A written `impl Default for &'static A` in a derived Default (ts-rust
// catch_conditional_refail_to_catch_if.rs `&'static ExportMember`): the
// derive glue calls it, also inside a tuple field.
use std::sync::atomic::{AtomicU32, Ordering};
pub struct ExportMember { pub name: &'static str, id: AtomicU32 }
static NONE: ExportMember = ExportMember { name: "<none>", id: AtomicU32::new(0) };
impl Default for &'static ExportMember {
    fn default() -> Self { println!("default member"); &NONE }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Methods {
    pub catch_method_name: &'static str,
    pub fail_method_name: &'static ExportMember,
    pub preferred_method_name: &'static str,
}
impl std::fmt::Debug for ExportMember {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "EM({}, {})", self.name, self.id.load(Ordering::Relaxed)) }
}
#[derive(Default, Debug)]
struct Outer { m: Methods, n: (u8, &'static ExportMember), o: Option<&'static ExportMember> }
fn main() {
    let m = Methods::default();
    println!("{:?}", m);
    let o = Outer::default();
    println!("{:?}", o);
    let e: &'static ExportMember = Default::default();
    println!("{}", e.name);
}
