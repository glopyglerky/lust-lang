// Lazy initializers that start with a block-like expression keep their
// meaning (the init fn body re-parses them).
thread_local! {
    static X: String = { "a" }.to_string();
    static Y: usize = if true { 1 } else { 2 } + 1;
    static Z: u8 = match 3 { 3 => 4, _ => 0 };
}
fn main() { X.with(|x| println!("{}", x)); Y.with(|y| println!("{}", y)); Z.with(|z| println!("{}", z)); }
