// The `u8` ascii predicates and case maps, checked over every byte
// value against rustc (W-BENCH4 rank 28). Prints one bitset per
// method so any range error changes the output.
fn bits(f: impl Fn(u8) -> bool) -> u64 {
    let mut h: u64 = 0;
    for b in 0..=255u8 {
        if f(b) {
            h = h.wrapping_mul(131).wrapping_add(b as u64 + 1);
        }
    }
    h
}
fn main() {
    println!("{}", bits(|b| b.is_ascii()));
    println!("{}", bits(|b| b.is_ascii_alphabetic()));
    println!("{}", bits(|b| b.is_ascii_alphanumeric()));
    println!("{}", bits(|b| b.is_ascii_digit()));
    println!("{}", bits(|b| b.is_ascii_whitespace()));
    println!("{}", bits(|b| b.is_ascii_uppercase()));
    println!("{}", bits(|b| b.is_ascii_lowercase()));
    println!("{}", bits(|b| b.is_ascii_hexdigit()));
    println!("{}", bits(|b| b.is_ascii_punctuation()));
    println!("{}", bits(|b| b.is_ascii_graphic()));
    println!("{}", bits(|b| b.is_ascii_control()));
    let mut up: u64 = 0;
    let mut lo: u64 = 0;
    for b in 0..=255u8 {
        up = up.wrapping_mul(131).wrapping_add(b.to_ascii_uppercase() as u64);
        lo = lo.wrapping_mul(131).wrapping_add(b.to_ascii_lowercase() as u64);
    }
    println!("{up} {lo}");
    println!("{} {}", b'A'.is_ascii_uppercase(), 0x80u8.is_ascii());
    const C: bool = b'z'.is_ascii_lowercase();
    const U: u8 = b'a'.to_ascii_uppercase();
    println!("{C} {}", U as char);
}
