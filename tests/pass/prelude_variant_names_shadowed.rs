// A crate fn named like a prelude variant shadows it, so a binding of
// that name is a plain binding; `None` in `if let` is the variant.
#[allow(non_snake_case)]
fn Some() -> i32 { 3 }
fn main() {
    #[allow(non_snake_case)]
    let Some = 5;
    println!("{}", Some);
    if let None = std::option::Option::Some(3) { println!("n"); } else { println!("s"); }
}
