// feature: core.numbers_format
// form: hex and binary literals, byte and char literals, raw strings, suffix literals
fn main() {
    let h = 0xFF;
    let b = 0b101;
    let byte = b'x';
    let bytes = b"ab";
    let raw = r#"a"b"#;
    println!("{} {} {} {} {} {} {}", h % 10, b, byte, bytes.len(), 'z' as u32 % 10, raw.len(), (1_000u64 % 7) as usize + 5usize);
}
