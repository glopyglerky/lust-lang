// feature: traits.impl_trait_arg
// form: fn arg as impl Display, formatted in the body
fn show(x: impl std::fmt::Display) -> String { format!("v={x}") }
fn main() { println!("{}", show(3)); }
