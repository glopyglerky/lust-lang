// Visibility edits on an `impl dyn Read` method in `mod api`, with `main`
// (the caller, in another module) unchanged on one warm daemon. A private
// or `pub(in crate::api)` method is rustc E0624 at the call; the warm
// caller must recheck when the method's visibility changes.
mod api {
    pub trait Read {
        fn read(&self) -> u32;
    }
    pub struct Number {
        pub n: u32,
    }
    impl Read for Number {
        fn read(&self) -> u32 {
            self.n
        }
    }
    impl dyn Read + '_ {
        fn secret(&self) -> u32 {
            self.read() + 30
        }
    }
}
fn main() {
    let n = api::Number { n: 7 };
    let d: &dyn api::Read = &n;
    println!("{}", d.secret());
}
