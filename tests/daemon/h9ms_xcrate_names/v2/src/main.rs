use util::{vec, format, concat, matches, stringify};
fn main() { let x = Some(1); println!("{:?} {} {} {} {}", vec![1,2], format!("b"), concat!("a","b"), matches!(x, Some(9)), stringify!(zz)); }
