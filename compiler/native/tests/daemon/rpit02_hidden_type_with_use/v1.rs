// impl Trait hidden type edits in a file with a `use` item (the resolver
// path): only the definer bodies change; the callers must see each new
// hidden type (hunt2 daemon_rpit_hidden_type_stale).
use std::fmt::Debug;
trait Val { fn v(&self) -> u64; }
struct A { x: u8 }
struct B { s: String, n: u64 }
impl Val for A { fn v(&self) -> u64 { self.x as u64 } }
impl Val for B { fn v(&self) -> u64 { self.n + self.s.len() as u64 } }
fn make(k: u64) -> impl Val { A { x: k as u8 } }
fn main() {
    let m = make(5);
    println!("{}", m.v());
}
