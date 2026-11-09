macro_rules! defm { () => { macro_rules! matches { ($e:expr, $p:pat) => { true }; } }; }
defm!();
fn main() { let x: Option<i32> = None; println!("{} {}", matches!(x, Some(9)), util::helper()); }
