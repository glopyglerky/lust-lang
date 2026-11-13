// ARCH 2.4 step 4: an impl param that the header does not fix is
// bound from an associated equality in the impl's own predicates,
// with no search. From the sol probe bound_only_param.rs.
trait A {
    type Item;
}
trait Tr {
    fn v() -> u8 {
        5
    }
}
impl<T, U> Tr for T where T: A<Item = U> {}
impl A for u8 {
    type Item = u16;
}
fn check<T: Tr>() -> u8 {
    T::v()
}
fn main() {
    println!("{}", check::<u8>());
}
