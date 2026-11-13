pub struct S;
impl S {
    const N: usize = 9;
    pub const P: usize = 1;
}
const fn hidden() -> usize { 1 }
pub const fn shown() -> usize { hidden() }
