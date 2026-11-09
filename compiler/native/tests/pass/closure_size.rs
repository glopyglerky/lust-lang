// A closure value is exactly its captures (ARCH 4.5).
fn size<T>(_x: &T) -> usize {
    std::mem::size_of::<T>()
}
fn main() {
    let a = 1u8;
    let b = 2u32;
    let c = 3u8;
    let none = || 5;
    let one_byte = move || a;
    let refs = || a as u32 + b + c as u32;
    let moved = move || a as u32 + b + c as u32;
    let s = String::from("x");
    let owns = move || s.len();
    println!("{} {} {} {} {}", size(&none), size(&one_byte), size(&refs), size(&moved), size(&owns));
    println!("{} {} {} {}", none(), one_byte(), refs(), moved() + owns() as u32);
}
