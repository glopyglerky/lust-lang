// `str::as_bytes` in a const fn body (ts-rust embed text hash).
const fn hash(b: &[u8]) -> u64 {
    let mut x = 0u64;
    let mut i = 0;
    while i < b.len() {
        x = x.wrapping_mul(31).wrapping_add(b[i] as u64);
        i += 1;
    }
    x
}
const fn text_hash(text: &str) -> u64 {
    if text.len() >= 3 { hash(text.as_bytes()) } else { 0 }
}
const H: u64 = text_hash("hello");
fn main() {
    let s = String::from("runtime");
    println!("{} {} {}", H, text_hash("ab"), text_hash(&s));
}
