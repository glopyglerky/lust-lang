// Generic fns as values through a qualified path: a generic impl's
// fn (`<W<u16> as Tr>::mk`), a trait default body (`<S as Tr>::twice`),
// a generic method with a turbofish, and an inherent fn of a generic
// impl (`<W<u16>>::get`). Each instantiates at the selected impl's
// args, as a call does, and coerces to a fn pointer or an Fn bound.
trait Tr {
    fn mk(x: u8) -> Self;
    fn get(&self) -> u32;
}
struct W<T>(T, u8);
impl<T: Default> Tr for W<T> {
    fn mk(x: u8) -> Self {
        W(T::default(), x)
    }
    fn get(&self) -> u32 {
        self.1 as u32 * 10
    }
}
trait Base {
    fn base(&self) -> u32;
    fn twice(&self) -> u32 {
        self.base() * 2
    }
}
struct S(u32);
impl Base for S {
    fn base(&self) -> u32 {
        self.0
    }
}
trait Conv {
    fn conv<U: From<u8>>(&self) -> U;
}
struct C(u8);
impl Conv for C {
    fn conv<U: From<u8>>(&self) -> U {
        U::from(self.0)
    }
}
trait Array {
    type Item;
    fn first(&self) -> Self::Item;
}
impl<T: Copy> Array for [T; 4] {
    type Item = T;
    fn first(&self) -> T {
        self[0]
    }
}
struct V<T>(T);
impl<T: Copy + std::ops::Add<Output = T>> V<T> {
    fn twice(&self) -> T {
        self.0 + self.0
    }
}
fn apply<F: Fn(&S) -> u32>(f: F) -> u32 {
    f(&S(5))
}
fn main() {
    let f = <W<u16> as Tr>::mk;
    let w = f(4);
    let g: fn(&W<u16>) -> u32 = <W<u16> as Tr>::get;
    let v: Vec<W<bool>> = vec![1u8, 2].into_iter().map(<W<bool> as Tr>::mk).collect();
    println!("{} {} {}", w.1, g(&w), v.len());
    let t = <S as Base>::twice;
    println!("{} {}", t(&S(21)), apply(<S as Base>::twice));
    let c = <C as Conv>::conv::<u32>;
    let fst = <[u8; 4] as Array>::first;
    let tw = <V<u16>>::twice;
    println!("{} {} {}", c(&C(7)), fst(&[9, 8, 7, 6]).wrapping_add(250), tw(&V(21)));
}
