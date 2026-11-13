// expect: L0602
// rustc E0666: `impl Trait` nested in the generic args of another `impl Trait`.
fn f(x: impl Into<Vec<impl std::fmt::Display>>) -> usize { x.into().len() }
fn main() { println!("{}", f(vec![1])); }
