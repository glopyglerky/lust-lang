// An inline const with no pinning type falls back to i32, not i128.
fn main() {
    let v = const { 5 };
    let w = const { 5u8 };
    println!("{} {}", std::mem::size_of_val(&v), std::mem::size_of_val(&w));
}
