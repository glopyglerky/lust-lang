// expect: L0104
// rustc E0599: next_power_of_two is an unsigned integer method.
fn main() { let x: i32 = 5; println!("{}", x.next_power_of_two()); }
