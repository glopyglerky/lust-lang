// `<Type>::item(..)` with no trait written resolves through the one trait in
// scope whose item it is and that holds for the type (rustc's path probe).
trait Tr { fn mk(&self) -> i32 { 3 } }
impl Tr for i32 {}
fn g<'a, T: Clone + Tr>(t: &'a T) -> &'a T { let _ = <T>::mk(t); <&T>::clone(&t) }
fn main() {
    println!("{} {} {}", g(&5i32), <i32>::clone(&6), <Vec<i32>>::clone(&vec![7]).len());
}
