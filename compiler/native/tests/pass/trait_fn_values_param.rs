// Trait fns as values through a type parameter, `Self`, a qualified
// path, and a type whose impl has no fn (derives, builtins).
use std::fmt::Debug;

trait Sh {
    fn area(&self) -> i32;
    fn mk(n: i32) -> Self
    where
        Self: Sized;
    fn dbl(&self) -> i32 {
        let f = Self::area;
        f(self) * 2
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
struct S(i32);

impl Sh for S {
    fn area(&self) -> i32 {
        self.0 * 2
    }
    fn mk(n: i32) -> Self {
        S(n)
    }
}

impl S {
    fn copies(&self) -> Vec<S> {
        let mut v = vec![self.clone()];
        v.resize_with(3, Self::default);
        let c = Self::clone;
        v.push(c(self));
        v
    }
}

trait Fill: Default + Debug {
    fn fill() -> Vec<Self> {
        let mut v = Vec::new();
        v.resize_with(2, Self::default);
        v
    }
}
impl Fill for i32 {}
impl Fill for S {}

fn area_of<T: Sh>(t: &T) -> i32 {
    let f = T::area;
    f(t)
}
fn make<T: Sh>() -> T {
    let f = <T as Sh>::mk;
    f(4)
}
fn areas<T: Sh>(v: &[T]) -> Vec<i32> {
    v.iter().map(T::area).collect()
}
fn clones<T: Clone>(v: &[T]) -> Vec<T> {
    v.iter().map(T::clone).collect()
}
fn qclones<T: Clone>(v: &[T]) -> Vec<T> {
    v.iter().map(<T as Clone>::clone).collect()
}
fn strings<T: std::fmt::Display>(v: &[T]) -> Vec<String> {
    v.iter().map(T::to_string).collect()
}
fn same<T: PartialEq>(a: &T, b: &T) -> bool {
    let f = T::eq;
    f(a, b)
}
fn order<T: Ord>(a: &T, b: &T) -> std::cmp::Ordering {
    let f = T::cmp;
    f(a, b)
}
fn apply<F: Fn(&i32, &i32) -> bool>(f: F) -> bool {
    f(&1, &1)
}

fn main() {
    println!("{} {}", area_of(&S(3)), S(4).dbl());
    let s: S = make();
    println!("{:?} {:?}", s, areas(&[S(1), S(2)]));
    println!("{:?} {:?} {:?}", clones(&[1, 2]), clones(&[S(5)]), qclones(&["a".to_string()]));
    println!("{:?}", strings(&[1.5, 2.0]));
    println!("{} {} {}", same(&1, &1), same(&S(1), &S(2)), same(&"a", &"a"));
    println!("{:?} {:?}", order(&S(1), &S(2)), order(&3, &2));
    println!("{:?} {:?} {:?}", S(7).copies(), i32::fill(), S::fill());
    let e = S::eq;
    let c = <i32 as Ord>::cmp;
    println!("{} {:?} {:?}", e(&S(1), &S(1)), c(&1, &2), i32::cmp(&2, &1));
    let d = [S(2), S(1)].iter().map(S::clone).collect::<Vec<_>>();
    let k: Vec<i32> = vec![&1, &2].into_iter().map(<i32 as Clone>::clone).collect();
    println!("{:?} {:?} {}", d, k, apply(i32::eq));
    let v = vec![1, 2, 3];
    println!("{:?} {:?}", v.iter().position(|x| i32::eq(x, &2)), v.binary_search_by(|x| i32::cmp(x, &2)));
    println!("{} {} {}", u8::max(3, 4), i32::min(1, 2), char::lt(&'a', &'b'));
    let mut w = vec![3, 1, 2];
    w.sort_by(i32::cmp);
    w.sort_by_key(i32::clone);
    let mut t = vec!["b", "a"];
    t.sort_by(|a, b| str::cmp(*a, *b));
    println!("{:?} {:?}", w, t);
    let lt: fn(&u8, &u8) -> bool = u8::lt;
    println!("{}", lt(&1, &2));
}
