// expect: L0510
mod pa {
    mod inner {
        pub(super) enum E {
            A,
            B,
        }
    }
    pub use self::inner::E::*;
    pub fn count() -> usize {
        [A, B].len()
    }
}
fn main() {
    let _ = pa::A;
    println!("{}", pa::count());
}
