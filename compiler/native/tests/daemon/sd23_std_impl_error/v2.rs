// sd23: a written `error` impl (plain form) removed and added again while main uses it
// through builtin glue; warm must equal cold and rustc (hunt11 21:29).
#[derive(Debug)]
struct S { a: u32 }
impl std::fmt::Display for S { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "E{}", self.a) } }
fn fail() -> Result<(), Box<dyn std::error::Error>> { Err(S { a: 9 })?; Ok(()) }
fn other() -> u32 { 1 }
fn main() { { let s = S { a: 1 }; let e: Box<dyn std::error::Error> = Box::new(s); println!("{}", e); } { let s = S { a: 1 }; println!("{} {}", fail().is_err(), s.a); } println!("{}", other()); }
