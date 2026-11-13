fn main() { let x = Some(1); println!("{} {}", matches!(x, Some(9)), util::matches!(x, Some(9))); }
