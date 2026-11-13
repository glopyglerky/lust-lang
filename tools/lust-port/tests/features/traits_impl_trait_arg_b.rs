// feature: traits.impl_trait_arg
// form: fn arg as impl Iterator with an item binding
// (an array by value, `[1, 2, 3].into_iter()`, is an iterators.chains gap)
fn total(it: impl Iterator<Item = i32>) -> i32 { it.sum() }
fn main() { println!("{}", total(vec![1, 2, 3].into_iter())); }
