// feature: traits.impl_trait_arg
// form: fn arg as impl Into<String>, as in ts-rust fn f(x: impl Into<FileText>)
fn take(s: impl Into<String>) -> usize { s.into().len() }
fn main() { println!("{}", take("abcd")); }
