// `Self` paths in a generic impl name the impl's Self type with its own
// args: variant calls and values, struct literals, the tuple struct
// constructor (called and as a value), and patterns.
#[derive(Debug)]
enum G<T> {
    V(T),
    N,
}
#[derive(Debug)]
struct W<T>(T, u8);
#[derive(Debug)]
struct S<T> {
    v: T,
}
struct U;
impl U {
    fn mk() -> Self {
        Self
    }
}
impl<T: Copy + std::fmt::Debug> G<T> {
    fn go(t: T) -> Self {
        Self::V(t)
    }
    fn none() -> Self {
        Self::N
    }
    fn get(&self) -> Option<T> {
        match self {
            Self::V(t) => Some(*t),
            Self::N => None,
        }
    }
}
impl<T: std::fmt::Debug> W<T> {
    fn mk(t: T) -> Self {
        Self(t, 1)
    }
    fn mk2(t: T) -> Self {
        let f = Self;
        f(t, 2)
    }
    fn first(self) -> T {
        let Self(a, _) = self;
        a
    }
}
impl<T> S<T> {
    fn mk(v: T) -> Self {
        Self { v }
    }
    fn take(self) -> T {
        let Self { v } = self;
        v
    }
}
fn main() {
    let g = G::go(4u16);
    println!("{:?} {:?} {:?}", g, G::<u8>::none(), g.get());
    println!("{:?} {:?} {}", W::mk("a"), W::mk2(3i8), W::mk(9u64).first());
    println!("{}", S::mk(5u8).take());
    let _u = U::mk();
}
