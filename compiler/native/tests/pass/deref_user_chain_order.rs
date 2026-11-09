// A field and a coercion through two user Deref steps (W -> N -> M):
// the outer impl runs first, as the prints show.
use std::ops::Deref;
struct M { v: u8 }
struct N { m: M }
impl Deref for N { type Target = M; fn deref(&self) -> &M { println!("N->M"); &self.m } }
struct W { n: N }
impl Deref for W { type Target = N; fn deref(&self) -> &N { println!("W->N"); &self.n } }
fn main() { let w = W { n: N { m: M { v: 9 } } }; println!("{}", w.v); let r: &M = &w; println!("{}", r.v); }
