// expect: L0602
// Valid Rust. Two instances of one generic impl Trait fn whose hidden closures capture the same types: lust cannot tell their closures apart yet.
use std::fmt::Display;
fn show<T: Default + Display>(tag: String) -> impl Fn() -> String { move || format!("{}{}", tag, T::default()) }
fn main() {
    println!("{} {}", show::<u32>(String::from("n"))(), show::<String>(String::from("s"))());
}
