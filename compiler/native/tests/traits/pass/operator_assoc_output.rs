// ARCH 2.5: an operator trait's Output associated type must
// normalize in the operator result type. Same shape as the
// ts-rust go_flags! BitOr impls (flags_macros.rs:53).
#[derive(Clone, Copy, PartialEq, Debug)]
struct Flags(u32);
impl std::ops::BitOr for Flags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Flags(self.0 | rhs.0)
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
struct Wide(u64);
impl std::ops::Add<Flags> for Wide {
    type Output = Wide;
    fn add(self, rhs: Flags) -> Wide {
        Wide(self.0 + rhs.0 as u64)
    }
}
fn main() {
    let f = Flags(1) | Flags(4);
    // The result type is Wide only through Add::Output.
    let w: Wide = Wide(10) + f;
    println!("{} {}", f.0, w.0);
}
