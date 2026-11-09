// sd23: a written `display` impl (plain form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
struct S { a: u32 }
impl std::fmt::Display for S { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "S{}", self.a) } }
fn other() -> u32 { 1 }
fn main() { { let s = S { a: 1 }; println!("{}", s); } { let s = S { a: 1 }; let t = format!("<{}>", s); println!("{}", t); } { let s = S { a: 1 }; println!("{s:>6}|"); } { let s = S { a: 1 }; let r = &s; println!("{}", r); } println!("{}", other()); }
