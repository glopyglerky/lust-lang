// Derive family proofs over a generic argument that is not an Adt
// path type: Debug and Default over `OnceCell<(..)>`, Default over
// `Cell<(..)>` and `Box<[T; N]>`, Eq and Hash over `Option<[T; N]>`
// (W-BENCH4 ranks 6, 10, 34).
use std::cell::{Cell, OnceCell};
#[derive(Clone, Debug, Default)]
struct P { ok: bool, v: OnceCell<(u8, Vec<String>)> }
#[derive(Default)]
struct Memo(Cell<(u64, i32, i32)>, Box<[u8; 2]>);
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
struct S { t: Option<[u32; 2]> }
fn main() {
    let p = P::default();
    println!("{:?}", p);
    p.v.set((1, vec!["x".to_string()])).unwrap();
    println!("{:?}", p.clone());
    let m = Memo::default();
    println!("{:?} {:?}", m.0.get(), m.1);
    let a = S { t: Some([1, 2]) };
    println!("{:?} {}", a, a == S::default());
}
