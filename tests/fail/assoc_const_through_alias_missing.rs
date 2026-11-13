// expect: L0104
// rustc E0599: the alias's Adt has no associated item `MISSING`.
pub struct M(i32);
impl M {
    pub const NONE: M = M(0);
}
pub type R = M;
fn main() {
    let _m = R::MISSING;
}
