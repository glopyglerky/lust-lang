// Impl where clauses that rustc accepts: a bound that fails before the
// growing one stops the evaluation (written order), and a where clause
// that names the impl's own trait ref is a cycle, not an overflow.
#![allow(dead_code)]
struct G<T>(T);
struct H<T>(T);
trait Tr {}
trait Miss {}
impl<T> Tr for G<T> where T: Miss, G<Option<T>>: Tr {}
impl<T: Miss> Tr for H<T> where H<Vec<T>>: Tr {}
trait Tc {}
impl<T> Tc for G<T> where G<T>: Tc {}
trait Tp {}
impl<T> Tp for G<T> {}
fn f<T>() where G<(T, T)>: Tp {}
fn main() {
    f::<u8>();
    println!("ok");
}
