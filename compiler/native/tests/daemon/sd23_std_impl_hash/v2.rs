// sd23: a written `hash` impl (plain form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
#[derive(PartialEq, Eq)]
struct S { a: u32 }
fn other() -> u32 { 1 }
fn main() { { let s = S { a: 1 }; let mut m = std::collections::HashSet::new(); m.insert(s); m.insert(S { a: 1 }); println!("{}", m.len()); } { let s = S { a: 1 }; let mut m = std::collections::HashMap::new(); m.insert(s, 4); println!("{:?}", m.get(&S { a: 1 })); } println!("{}", other()); }
