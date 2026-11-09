// More shapes the template check must accept: generic inherent impls
// with bounds, default bodies with where clauses, ?Sized params by
// reference, recursion, iterator bounds, and slices of params.
use std::fmt::Debug;
use std::fmt::Display;
#[derive(Debug, Clone)]
struct W<T> { x: T }
impl<T: Clone> W<T> {
    fn get(&self) -> T { self.x.clone() }
    fn dup(&self) -> (T, T) { (self.get(), self.get()) }
}
impl<T: Display> W<T> {
    fn show(&self) -> String { format!("<{}>", self.x) }
}
trait Named { fn name(&self) -> String; fn greet(&self) -> String where Self: Debug { format!("{:?} {}", self, self.name()) } }
#[derive(Debug)]
struct N(u8);
impl Named for N { fn name(&self) -> String { format!("n{}", self.0) } }
fn outer<T: Debug>(t: T) -> String { inner(&t) }
fn inner<U: Debug + ?Sized>(u: &U) -> String { format!("{:?}", u) }
fn count<T: Copy>(n: u32, t: T) -> u32 { if n == 0 { 0 } else { 1 + count(n - 1, t) } }
fn first<T: Clone>(v: &Vec<T>) -> T { v[0].clone() }
fn unwrap_or<T>(o: Option<T>, d: T) -> T { match o { Some(x) => x, None => d } }
fn sum_iter<I: Iterator<Item = u32>>(mut it: I) -> u32 { let mut s = 0; while let Some(x) = it.next() { s += x; } s }
struct Up(u32);
impl Iterator for Up { type Item = u32; fn next(&mut self) -> Option<u32> { if self.0 >= 3 { None } else { self.0 += 1; Some(self.0) } } }
fn twice_show<T: Display + Clone>(t: &T) -> String { let u = t.clone(); format!("{}{}", t, u) }
fn eq_all<T: PartialEq>(a: &[T], b: &T) -> bool { let mut i = 0; while i < a.len() { if a[i] != *b { return false; } i += 1; } true }
fn main() {
    let w = W { x: String::from("s") };
    println!("{:?} {}", w.dup(), w.show());
    println!("{}", N(4).greet());
    println!("{}", outer(vec![1u8, 2]));
    println!("{}", count(5, 'c'));
    println!("{}", first(&vec![String::from("a"), String::from("b")]));
    println!("{} {}", unwrap_or(Some(1), 2), unwrap_or(None, 3));
    println!("{}", sum_iter(Up(0)));
    println!("{}", twice_show(&7u8));
    println!("{} {}", eq_all(&[1u8, 1], &1), eq_all(&[1u8, 2], &1));
}
