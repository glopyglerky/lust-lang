// A path import of a macro re-export (`macro_rules!` + `pub(crate) use`)
// from a module declared later in the file (ts-rust impl_es_transformer).
mod user {
    use crate::m2::k;
    pub fn f() -> u32 {
        k!() + crate::m2::k!()
    }
}
mod m2 {
    macro_rules! k {
        () => {
            1u32
        };
    }
    pub(crate) use k;
}
fn main() {
    println!("{}", user::f());
}
