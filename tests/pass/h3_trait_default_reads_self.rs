// A trait default const that reads `Self` takes each impl's Self type:
// `D = Self::C + 2` is 102 for S, while T2 writes its own D; the default
// also works through a supertrait and a generic fn.
trait Tr { const C: u8; const D: u8 = Self::C + 2; }
struct S;
struct T2;
impl Tr for S { const C: u8 = 100; }
impl Tr for T2 { const C: u8 = 1; const D: u8 = 9; }
impl Tr for u16 { const C: u8 = 5; }
trait A { const K: i32; }
trait B: A { const KN: i32 = Self::K * 2; fn b(&self) -> i32 { Self::KN } }
impl A for S { const K: i32 = 21; }
impl B for S {}
static K: u8 = S::D;
fn g<X: Tr>() -> u8 { X::D }
fn main() {
    println!("{} {} {} {} {}", S::D, T2::D, K, g::<u16>(), <S as Tr>::D);
    println!("{} {} {}", S.b(), S::KN, more());
}
// A generic impl takes the default too (evaluated per instance).
struct W<T>(T);
impl<T> Tr for W<T> { const C: u8 = 3; }
const J: u8 = <S as Tr>::D;
fn more() -> u8 { W::<u8>::D + J }
