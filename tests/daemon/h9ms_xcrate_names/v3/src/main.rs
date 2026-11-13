use util::{vec, format, concat, matches, stringify};
fn main() { let x = Some(2); println!("{:?} {} {} {} {}", vec![7], format!("b"), concat!("a","b"), matches!(x, Some(9)), stringify!(zz)); }
