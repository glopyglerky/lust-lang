// sd23: a written `deref` impl (macro form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
struct S { a: u32 }
macro_rules! imp { () => { impl std::ops::Deref for S { type Target = u32; fn deref(&self) -> &u32 { &self.a } } }; }
imp!();
fn other() -> u32 { 1 }
fn main() { { let s = S { a: 1 }; println!("{}", *s); } { let s = S { a: 1 }; println!("{}", s.count_ones()); } { let s = S { a: 1 }; let r: &u32 = &s; println!("{}", r); } println!("{}", other()); }
