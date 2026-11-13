// sd23: a written `partialeq` impl (plain form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
#[derive(Debug)]
struct S { a: u32 }
fn other() -> u32 { 2 }
fn main() { { let s = S { a: 1 }; println!("{}", s == S { a: 1 }); } { let s = S { a: 1 }; println!("{}", s != S { a: 2 }); } { let s = S { a: 1 }; assert_eq!(s, S { a: 1 }); println!("ok"); } { let s = S { a: 1 }; assert_ne!(s, S { a: 3 }); println!("ok"); } { let s = S { a: 1 }; println!("{}", Some(s) == None); } { let s = S { a: 1 }; println!("{}", [s] == [S { a: 1 }]); } println!("{}", other()); }
