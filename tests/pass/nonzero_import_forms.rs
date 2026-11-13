// NonZero names by explicit import, by glob (module and block), by full
// path, and through a module re-export. A fn-body `use self::*;` is valid.
use std::num::*;
mod m {
    pub use std::num::NonZero;
}
mod k {
    use std::num::NonZeroU16;
    pub fn f() -> u16 {
        NonZeroU16::new(7).unwrap().get()
    }
}
fn main() {
    use self::*;
    let a = NonZero::new(5u32).unwrap();
    let b = NonZeroU8::MIN;
    let c = std::num::NonZeroI64::new(-3).unwrap();
    let d = m::NonZero::new(9u64).unwrap();
    let e = {
        use core::num::NonZeroUsize;
        NonZeroUsize::new(4).map(|x| x.get())
    };
    println!("{} {} {} {} {:?} {}", a, b, c, d, e, k::f());
}
