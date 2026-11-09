// feature: macros.std_missing
// form: matches!, debug_assert! and the vec! repeat form
fn main() {
    debug_assert!(1 + 1 == 2);
    let v = vec![9; 2];
    println!("{} {}", matches!(Some(1), Some(_)), v.len() + v[0]);
}
