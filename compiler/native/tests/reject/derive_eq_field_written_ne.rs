// expect: L0112
// Valid Rust: a field type whose written PartialEq also writes ne is
// called through ne by core (tuple !=); the eq glue does not model that.
#[derive(Debug)]
struct N(u8);
impl PartialEq for N { fn eq(&self, o: &N) -> bool { println!("eq"); self.0 == o.0 } fn ne(&self, o: &N) -> bool { println!("ne"); self.0 != o.0 } }
#[derive(PartialEq)]
struct W { n: N }
fn main() { println!("{} {}", (N(1), 2) != (N(1), 3), W { n: N(1) } != W { n: N(2) }); }
