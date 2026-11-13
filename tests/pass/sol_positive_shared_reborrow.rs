// Sol review probe `positive_shared_reborrow`: rustc accepts.
fn good<'a, 'b>(x: &'a &'b i32) -> &'b i32 { &**x }
fn main() { let x = 1; println!("{}", good(&&x)); }
