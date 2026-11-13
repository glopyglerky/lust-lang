// Generic trait methods, trait inputs, supertraits, assoc-type returns, and
// default bodies through bounds: each calls the trait method even when the
// concrete type has an inherent method with the same name.
struct S(u8);
impl S {
    fn m<U: Copy>(&self, u: U) -> U { u }
    fn conv(&self) -> u16 { 999 }
    fn base(&self) -> u8 { 77 }
    fn get(&self) -> u8 { 66 }
    fn first(&self) -> u8 { 55 }
}
trait Tr { fn m<U: Copy>(&self, u: U) -> U; }
impl Tr for S { fn m<U: Copy>(&self, u: U) -> U { let _ = self.0; u } }
trait Conv<T> { fn conv(&self) -> T; }
impl Conv<u16> for S { fn conv(&self) -> u16 { self.0 as u16 + 1 } }
trait Base { fn base(&self) -> u8; }
trait Sub: Base { fn sub(&self) -> u8 { self.base() + 1 } }
impl Base for S { fn base(&self) -> u8 { self.0 } }
impl Sub for S {}
trait It { type Item; fn get(&self) -> Self::Item; }
impl It for S { type Item = u8; fn get(&self) -> u8 { self.0 * 2 } }
trait Seq { fn first(&self) -> u8; fn two(&self) -> u8 { self.first() + self.first() } }
impl Seq for S { fn first(&self) -> u8 { self.0 } }
fn a<T: Tr>(t: &T) -> u32 { t.m(7u32) }
fn b<T: Conv<u16>>(t: &T) -> u16 { t.conv() }
fn c<T: Sub>(t: &T) -> u8 { t.base() + t.sub() }
fn d<T: It<Item = u8>>(t: &T) -> u8 { t.get() }
fn e<T: Seq>(t: &T) -> u8 { t.two() }
struct W<T>(T);
impl<T: Tr + Conv<u16>> W<T> { fn both(&self) -> u32 { self.0.m(3u32) + self.0.conv() as u32 } }
fn main() {
    let s = S(5);
    println!("{} {} {} {} {} {}", a(&s), b(&s), c(&s), d(&s), e(&s), W(S(2)).both());
    println!("{} {} {} {} {}", s.m(1u8), s.conv(), s.base(), s.get(), s.first());
}
