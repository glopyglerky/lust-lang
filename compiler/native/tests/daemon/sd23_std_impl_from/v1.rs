// sd23: a written `from` impl (plain form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
struct S { a: u32 }
fn mk() -> Result<u32, S> { Err(7u32)?; Ok(0) }
impl From<u32> for S { fn from(a: u32) -> S { S { a } } }
fn other() -> u32 { 1 }
fn main() { { let s = S { a: 1 }; println!("{} {}", S::from(4).a, s.a); } { let s = S { a: 1 }; let t: S = 5.into(); println!("{} {}", t.a, s.a); } { let s = S { a: 1 }; println!("{} {}", match mk() { Err(e) => e.a, Ok(v) => v }, s.a); } println!("{}", other()); }
