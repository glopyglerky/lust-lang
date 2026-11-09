// Const and static items read trait associated consts at a concrete Self
// type: `S::C` through a trait in scope, `<S as Tr>::C` by name, trait
// defaults, primitive Self types, and array lengths.
mod m {
    pub trait Named { const NAME: &'static str; }
    impl Named for super::S { const NAME: &'static str = "s"; }
}
use m::Named;
trait Tr { const C: i32; const D: i32 = 5; }
trait Len { const N: usize; }
struct S;
impl Tr for S { const C: i32 = 7; }
impl Tr for u8 { const C: i32 = 2; const D: i32 = 6; }
impl Len for S { const N: usize = 3; }
static K: i32 = S::C;
const J: i32 = <S as Tr>::C + 1;
const P: i32 = u8::C * <u8 as Tr>::D + S::D;
const N: &str = S::NAME;
fn main() {
    static L: i32 = <S as Tr>::D;
    let a = [0u8; <S as Len>::N];
    println!("{} {} {} {} {} {:?}", K, J, P, N, L, a);
}
