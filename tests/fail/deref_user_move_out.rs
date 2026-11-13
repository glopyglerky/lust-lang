// expect: L0305
// rustc E0507: a by-value method through a user `Deref` moves out of the dereference.
use std::ops::Deref;
struct N { s: String }
impl N { fn take(self) -> String { self.s } }
struct W { n: N }
impl Deref for W { type Target = N; fn deref(&self) -> &N { &self.n } }
fn main() { let w = W { n: N { s: String::from("x") } }; println!("{}", w.take()); }
