fn take(b: &[u8]) -> u8 {
    b[0]
}
static SB: &[u8; 3] = b"a\xffz";
fn main() {
    let b = b"hi\n";
    println!("{} {} {}", b.len(), b[0], b[2]);
    println!("{}", b'A' + 1);
    println!("{} {}", b'\x41', b'\n');
    let r = br#"a"b"#;
    println!("{} {}", r.len(), r[1]);
    let e: &[u8; 0] = b"";
    println!("{}", e.len());
    println!("{} {}", take(b"xy"), take(SB));
    println!("{} {}", SB[1], SB.len());
    let mut sum = 0u32;
    for i in 0..b.len() {
        sum += b[i] as u32;
    }
    println!("{}", sum);
    println!("{}", b == b"hi\n");
}
