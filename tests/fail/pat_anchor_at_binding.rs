// expect: L0001
// rustc: the left side of `@` must be a binding; a path is not one.
mod k {
    pub enum E {
        A(u32),
    }
}
fn main() {
    let _ = match k::E::A(1) {
        crate::k::E::A @ _ => 1,
        _ => 2,
    };
}
