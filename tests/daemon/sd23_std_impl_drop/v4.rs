// sd23: a written `drop` impl (plain form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
struct S { a: u32 }
struct W { s: S }
fn other() -> u32 { 2 }
fn main() { { let s = S { a: 1 }; { let _t = S { a: 2 }; } println!("{}", s.a); } { let s = S { a: 1 }; let v = vec![s]; drop(v); println!("after"); } { let s = S { a: 1 }; std::mem::drop(s); println!("after"); } { let s = S { a: 1 }; let w = W { s }; println!("{}", w.s.a); } println!("{}", other()); }
