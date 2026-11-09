// feature: traits.impl_trait_arg
// form: impl Trait in an item binding, as in ts-rust Diagnostic::with_arguments
fn args(xs: impl IntoIterator<Item = impl Into<String>>) -> Vec<String> { xs.into_iter().map(|x| x.into()).collect() }
fn main() { println!("{:?} {:?}", args(vec!["a", "b"]), args(vec![String::from("c")])); }
