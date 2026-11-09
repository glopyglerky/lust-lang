// sd23: a written `lowerhex` impl (plain form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
struct S { a: u32 }
fn other() -> u32 { 1 }
fn main() { { let s = S { a: 1 }; println!("{:x}", s); } { let s = S { a: 1 }; let t = format!("<{:x}>", s); println!("{}", t); } println!("{}", other()); }
