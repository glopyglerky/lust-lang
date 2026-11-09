use util::alwaystrue as matches;
fn main() { let x = Some(1); println!("{}", matches!(x, Some(9))); }
