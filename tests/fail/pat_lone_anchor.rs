// expect: L0001
// rustc E0532: a lone `crate` is not a pattern (not a binding).
mod k {
    pub enum E {
        A,
    }
}
fn main() {
    let _ = match k::E::A {
        crate => 1,
    };
}
