// sd23: a written `clone` impl (plain form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
struct S { a: u32 }
#[derive(Clone)]
struct T2 { s: S }
impl Clone for S { fn clone(&self) -> S { S { a: self.a + 10 } } }
fn other() -> u32 { 1 }
fn main() { { let s = S { a: 1 }; println!("{}", s.clone().a); } { let s = S { a: 1 }; let t = T2 { s }; println!("{}", t.clone().s.a); } { let s = S { a: 1 }; let v = vec![s]; println!("{}", v.clone()[0].a); } { let s = S { a: 1 }; let o = Some(s); println!("{}", o.clone().unwrap().a); } { let s = S { a: 1 }; let v = vec![s; 2]; println!("{}", v[1].a); } println!("{}", other()); }
