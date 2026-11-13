// expect: L0611
// rustc E0275 at the use: `T: Miss` fails first at the impl (so the impl
// itself is fine), but proving `G<u8>: Tr` still asks `G<(u8, u8)>: Tr`
// and so on. lust hung here: the no-impl message printed the doubling type.
struct G<T>(T);
trait Tr {}
trait Miss {}
impl<T> Tr for G<T> where T: Miss, G<(T, T)>: Tr {}
fn need<X: Tr>(_x: &X) -> u32 {
    1
}
fn main() {
    println!("{}", need(&G(2u8)));
}
