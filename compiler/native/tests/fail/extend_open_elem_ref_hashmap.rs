// expect: L0610
// rustc E0282: HashMap has Extend<(K, V)> and Extend<(&K, &V)>.
use std::collections::HashMap;
fn main() {
    let (k, x) = (1u8, 'x');
    let mut v = HashMap::new();
    v.extend([(&k, &x)]);
    println!("{}", v.len());
}
