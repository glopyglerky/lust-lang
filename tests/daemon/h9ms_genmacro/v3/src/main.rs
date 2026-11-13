macro_rules! defm { () => { macro_rules! matches { ($e:expr, $p:pat) => { true }; } }; }
defm!();
fn main() { let x = Some(1); println!("{} {}", matches!(x, Some(9)), util::helper()); }
