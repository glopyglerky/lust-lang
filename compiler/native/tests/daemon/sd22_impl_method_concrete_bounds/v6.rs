// Where clauses over concrete types on methods of a blanket impl (a template) and a
// concrete impl: collect proves non-structural ones with the impls it has at that point.
mod m { pub const R: u32 = 1; }
pub trait Val {
    fn val(&self) -> u32;
}
impl<T: Copy> Val for T {
    fn val(&self) -> u32 {
        7
    }
}
struct S;
impl Val for S {
    fn val(&self) -> u32 {
        8
    }
}
fn wrap<T: Copy>(t: T) -> T { t }
fn main() { println!("{}", 5i32.val() + S.val() + m::R + wrap(1)); }
