// expect: L0541
fn main() { #[allow(unconditional_panic)] println!("{}", 1 / 0); }
