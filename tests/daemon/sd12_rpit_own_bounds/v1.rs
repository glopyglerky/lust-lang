use std::fmt::Debug;
fn pick<T: Copy + Debug>(x: T) -> T { x }
struct S { v: Vec<u32> }
impl S {
    fn vals(&self) -> impl Iterator<Item = &u32> {
        self.v.iter()
    }
}
fn evens(n: u32) -> impl Iterator<Item = u32> { (0..n).filter(|x| x % 2 == 0) }
fn user(s: &S) -> u32 { s.vals().map(|x| pick(*x)).sum::<u32>() + evens(5).sum::<u32>() }
fn main() { let s = S { v: vec![1, 2] }; println!("{}", user(&s)); }
