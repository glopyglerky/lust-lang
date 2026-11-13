// `impl Trait` inside an associated type binding of an `impl Trait`
// argument is one more anonymous type param (rustc allows it there).
fn args(xs: impl IntoIterator<Item = impl Into<String>>) -> Vec<String> { let mut v = Vec::new(); for x in xs { v.push(x.into()); } v }
fn args2(xs: impl IntoIterator<Item = impl Into<String>>) -> Vec<String> { xs.into_iter().map(|x| x.into()).collect() }
fn shows(xs: impl Iterator<Item = impl std::fmt::Display>) -> String { xs.map(|x| format!("<{}>", x)).collect() }
fn main() {
    println!("{:?}", args(vec!["a", "b"]));
    println!("{:?}", args(vec![String::from("c")]));
    println!("{:?}", args2(vec!["d"]));
    println!("{}", shows(vec![1, 2].into_iter()));
}
