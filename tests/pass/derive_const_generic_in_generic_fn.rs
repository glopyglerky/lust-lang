// Derived impls of a const-generic type hold for its own parameter: a
// generic fn over `const N` uses `A<N>: PartialEq + Eq + Clone`, and a
// derive on a type with an `A<N>` field holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct A<const N: usize> {
    bytes: [u8; N],
    len: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind<const N: usize> {
    Array(A<N>),
    Static(&'static str),
}

fn same<T: Eq + Clone>(t: &T) -> bool {
    *t == t.clone()
}

fn check<const N: usize>(a: A<N>) -> bool {
    same(&a) && same(&Kind::Array(a))
}

fn main() {
    let a = A { bytes: [1, 2, 3], len: 3 };
    println!("{} {}", check(a), check(A { bytes: [], len: 0 }));
    println!("{:?} {}", Kind::<2>::Static("x"), Kind::<1>::Static("a") < Kind::Array(A { bytes: [0], len: 1 }));
}
