// ARCH 2.2 derives: each derived impl bounds every type parameter by the
// derived trait (rustc), except Default on an enum, whose #[default]
// variant is a unit variant: E<NoDefault> still has Default.
use std::hash::Hash;
fn need<T: Hash + Ord + Default + std::fmt::Debug>(t: &T) -> String {
    format!("{:?} {:?}", t, t.cmp(&T::default()))
}
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
struct W<T> { t: T, n: u8 }
struct NoDefault;
#[derive(Default)]
enum E<T> { #[default] A, B(T) }
fn main() {
    println!("{}", need(&W { t: 3u8, n: 1 }));
    println!("{}", need(&W { t: String::from("x"), n: 0 }));
    let e: E<NoDefault> = E::default();
    println!("{}", matches!(e, E::A));
}
