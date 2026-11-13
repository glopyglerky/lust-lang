// ARCH 2.1 and 2.4 step 1: a bound on Sub puts Super in the param
// env by elaboration, so a body bounded only by Sub can call Super.
trait Super {
    fn base(&self) -> u8;
}
trait Sub: Super {
    fn extra(&self) -> u8;
}
struct S;
impl Super for S {
    fn base(&self) -> u8 {
        10
    }
}
impl Sub for S {
    fn extra(&self) -> u8 {
        5
    }
}
// Only Sub is written here. base() must resolve from elaboration.
fn both<T: Sub>(v: &T) -> u8 {
    v.base() + v.extra()
}
// A Super bound is also met by a Sub value's type at a call site.
fn need_super<T: Super>(v: &T) -> u8 {
    v.base()
}
fn main() {
    println!("{} {}", both(&S), need_super(&S));
}
