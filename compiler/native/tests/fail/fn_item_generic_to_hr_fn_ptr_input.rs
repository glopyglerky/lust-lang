// expect: L0110
// rustc E0308: a generic fn's type param cannot take the region of the
// fn pointer's elided input lifetime, even when the output has none.
fn first<T>(_x: T) -> u8 {
    3
}
fn main() {
    let g: fn(&u8) -> u8 = first;
    println!("{}", g(&1));
}
