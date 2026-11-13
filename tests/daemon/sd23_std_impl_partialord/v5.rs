// sd23: a written `partialord` impl (macro form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
#[derive(PartialEq)]
struct S { a: u32 }
macro_rules! imp { () => { impl PartialOrd for S { fn partial_cmp(&self, o: &S) -> Option<std::cmp::Ordering> { self.a.partial_cmp(&o.a) } } }; }
imp!();
fn other() -> u32 { 3 }
fn main() { { let s = S { a: 1 }; println!("{}", s < S { a: 2 }); } { let s = S { a: 1 }; println!("{}", s >= S { a: 1 }); } { let s = S { a: 1 }; println!("{}", s.partial_cmp(&S { a: 0 }) == Some(std::cmp::Ordering::Greater)); } println!("{}", other()); }
