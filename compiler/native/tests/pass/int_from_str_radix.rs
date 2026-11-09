// uN::from_str_radix (TRIAGE R26: the codegen tools' JSON hex4 escape)
// at radixes 2 to 36, every width, both letter cases, with core's error
// kinds and its overflow-before-bad-digit order.
fn hex4(text: &str, pos: usize) -> Option<u32> {
    let digits = text.get(pos..pos + 4);
    digits.filter(|d| d.bytes().all(|b| b.is_ascii_hexdigit())).and_then(|d| u32::from_str_radix(d, 16).ok())
}
fn main() {
    println!("{:?} {:?} {:?}", u32::from_str_radix("ff", 16), i8::from_str_radix("-80", 16), u8::from_str_radix("1_0", 2));
    println!("{:?} {:?} {:?}", i64::from_str_radix("-zz", 36), u128::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", 16), u128::from_str_radix("100000000000000000000000000000000", 16));
    println!("{:?} {:?} {:?} {:?}", u16::from_str_radix("", 8), i32::from_str_radix("-", 10), u8::from_str_radix("-1", 10), i8::from_str_radix("-81", 16));
    println!("{:?} {:?} {:?}", u8::from_str_radix("1G", 16), u8::from_str_radix("ZZZ9", 36), usize::from_str_radix("777", 8));
    let s = String::from("101");
    println!("{:?} {:?}", u32::from_str_radix(&s, 2), hex4("x00e9y", 1));
    let parsed: Vec<u8> = ["7", "f"].iter().filter_map(|t| u8::from_str_radix(t, 16).ok()).collect();
    println!("{:?}", parsed);
}
