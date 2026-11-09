// expect: L0112
// Valid Rust. `<` on a slice whose element has a written PartialOrd beside an Ord impl: lust
// builds only the `partial_cmp` method for these types, not the operators or `lt`.
use std::cmp::Ordering;
#[derive(PartialEq, Eq, Clone, Copy)]
struct Name(u32);
impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}
impl Ord for Name {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
fn main() {
    let a = [Name(1)];
    let b = [Name(2)];
    println!("{}", a[..] < b[..]);
}
