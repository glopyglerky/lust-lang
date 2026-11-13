// Different fn items in an array join as fn pointers; one fn item
// repeated keeps its zero-sized item type (size_of_val 0, as rustc).
fn a(x: u8) -> u8 { x + 1 }
fn b(x: u8) -> u8 { x * 2 }
fn c(x: u16) -> u8 { x as u8 }
fn main() {
    let v = [a, a];
    let w = [a, b, a];
    let p: fn(u8) -> u8 = b;
    let z = [a, p];
    println!("{} {} {} {} {}", std::mem::size_of_val(&v), (v[1])(2), std::mem::size_of_val(&w), (w[1])(3), (z[1])(4));
}
