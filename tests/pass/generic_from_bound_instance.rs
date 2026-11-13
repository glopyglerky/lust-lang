// `U::from(7)` under `U: From<u8>`: the literal is a u8 by the bound,
// in every instance too (a builtin Self has several From impls).
struct M(u8);
impl From<u8> for M {
    fn from(x: u8) -> M {
        M(x + 1)
    }
}
fn mk<U: From<u8>>() -> U {
    U::from(7)
}
struct S;
impl S {
    fn conv<U: From<u8>>(&self) -> U {
        U::from(200)
    }
}
fn main() {
    let a: u16 = mk();
    let b: i64 = mk();
    let c: M = mk();
    let d: u32 = S.conv();
    let f = S::conv::<u64>;
    println!("{} {} {} {} {}", a, b, c.0, d, f(&S));
}
