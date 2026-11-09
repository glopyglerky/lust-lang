// `Self::f` in a concrete impl (`impl S<u8>`, `impl Tr for S<u8>`) calls a
// fn of the generic impl at Self's args, so literals take Self's types.
struct S<T>(T);
impl<T> S<T> {
    fn mk(x: T) -> S<T> {
        S(x)
    }
    fn zero() -> usize {
        0
    }
}
impl S<u8> {
    fn g() -> usize {
        std::mem::size_of_val(&Self::mk(7)) + Self::zero()
    }
}
trait Tr {
    fn h() -> usize;
}
impl Tr for S<u16> {
    fn h() -> usize {
        std::mem::size_of_val(&Self::mk(7).0)
    }
}
impl<T: Copy> S<T> {
    fn dup(&self) -> S<T> {
        Self::mk(self.0)
    }
}
// A partly concrete impl: `Self` is `C<T, 2>`.
struct C<T, const N: usize>(T);
impl<T: Default, const N: usize> C<T, N> {
    fn mk(x: T) -> Self {
        C(x)
    }
    fn n() -> usize {
        N
    }
}
impl<T: Default> C<T, 2> {
    fn g(x: T) -> usize {
        std::mem::size_of_val(&Self::mk(x)) + Self::n()
    }
}
fn main() {
    println!("{} {} {} {}", S::<u8>::g(), <S<u16> as Tr>::h(), S(5u64).dup().0, C::<u16, 2>::g(5));
}
