// sd23: a written `iterator` impl (plain form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
struct S { a: u32 }
impl Iterator for S { type Item = u32; fn next(&mut self) -> Option<u32> { if self.a < 4 { self.a += 1; Some(self.a) } else { None } } }
fn other() -> u32 { 1 }
fn main() { { let s = S { a: 1 }; for x in s { println!("{}", x); } } { let s = S { a: 1 }; println!("{}", s.sum::<u32>()); } { let s = S { a: 1 }; let v: Vec<u32> = s.collect(); println!("{:?}", v); } { let s = S { a: 1 }; let mut s = s; println!("{:?}", s.next()); } { let s = S { a: 1 }; for (i, x) in s.enumerate() { println!("{} {}", i, x); } } println!("{}", other()); }
