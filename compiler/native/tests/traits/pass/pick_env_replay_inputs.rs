// ARCH 4.1: a template call proved by a where clause resolves once;
// instances replay the clause with its trait inputs. The concrete
// Self types here have a second impl (`From<u16>`, `Add2<u16>`,
// `PartialEq<u16>`) that a fresh re-selection would trip on.
trait Add2<X> {
    fn add2(&self, x: X) -> u32;
}
struct S;
impl Add2<u8> for S {
    fn add2(&self, x: u8) -> u32 {
        x as u32 + 1000
    }
}
impl Add2<u16> for S {
    fn add2(&self, x: u16) -> u32 {
        x as u32 + 2000
    }
}
fn go<U: Add2<u8>>(u: U) -> u32 {
    u.add2(8)
}
struct W(u64);
impl From<u8> for W {
    fn from(x: u8) -> W {
        W(x as u64 + 100)
    }
}
impl From<u16> for W {
    fn from(x: u16) -> W {
        W(x as u64 + 200)
    }
}
fn conv<U: From<u8>>() -> U {
    From::from(8)
}
struct M(u32);
impl PartialEq<u8> for M {
    fn eq(&self, o: &u8) -> bool {
        self.0 == *o as u32
    }
}
impl PartialEq<u16> for M {
    fn eq(&self, o: &u16) -> bool {
        self.0 == *o as u32 + 1
    }
}
fn eq5<T: PartialEq<u8>>(t: T) -> bool {
    t == 5
}
fn main() {
    let a: u16 = conv();
    let w: W = conv();
    println!("{} {} {} {} {}", go(S), a, w.0, eq5(M(5)), eq5(M(6)));
}
