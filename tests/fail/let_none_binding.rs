// expect: L0110
// rustc E0308: `None` here is the std variant, not a new binding.
fn main() {
    let None = 5;
    println!("{}", None);
}
