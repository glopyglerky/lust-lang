// sd23: a written `default` impl (module form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
struct S { a: u32 }
#[derive(Default)]
struct T2 { s: S }
mod m { use super::S; impl Default for S { fn default() -> S { S { a: 7 } } } }
fn other() -> u32 { 1 }
fn main() { { let s = S { a: 1 }; let d = S::default(); println!("{} {}", d.a, s.a); } { let s = S { a: 1 }; let d: S = Default::default(); println!("{} {}", d.a, s.a); } { let s = S { a: 1 }; let d = S { ..Default::default() }; println!("{} {}", d.a, s.a); } { let s = S { a: 1 }; let d: Option<S> = None; println!("{} {}", d.unwrap_or_default().a, s.a); } { let s = S { a: 1 }; let d = T2::default(); println!("{} {}", d.s.a, s.a); } println!("{}", other()); }
