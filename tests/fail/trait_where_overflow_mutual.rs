// expect: L0611
// rustc E0275: the growth goes through a second trait and a blanket impl.
struct G<T>(T);
trait Tr {}
trait Tb {}
impl<T> Tr for G<T> where (T, T): Tb {}
impl<T> Tb for T where G<T>: Tr {}
fn main() {
    println!("ok");
}
