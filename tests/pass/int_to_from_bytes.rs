// Integer byte conversions (TRIAGE R26): to_{le,be,ne}_bytes and
// from_{le,be,ne}_bytes on every width, by path and as fn values
// (ts-rust's hashing and binary readers), and in a const item.
fn le32(b: &[u8]) -> u32 { u32::from_le_bytes(b[..4].try_into().unwrap()) }
fn main() {
    let x: u32 = 0x12345678;
    println!("{:?} {:?} {:?}", x.to_le_bytes(), x.to_be_bytes(), x.to_ne_bytes());
    println!("{} {} {}", u32::from_le_bytes([1, 2, 3, 4]), u32::from_be_bytes([1, 2, 3, 4]), u32::from_ne_bytes([1, 2, 3, 4]));
    println!("{:?} {}", (-2i16).to_be_bytes(), i16::from_le_bytes([0xfe, 0xff]));
    println!("{:?} {}", u64::MAX.to_le_bytes(), u64::from_le_bytes(7u64.to_le_bytes()));
    println!("{:?}", (-1i128 << 70).to_be_bytes());
    println!("{} {}", i128::from_le_bytes((-5i128).to_le_bytes()), u128::from_be_bytes([9; 16]));
    println!("{:?} {}", 300usize.to_le_bytes(), isize::from_be_bytes([0xff; 8]));
    println!("{:?} {}", 200u8.to_be_bytes(), i8::from_le_bytes([0x80]));
    let mut out = Vec::new();
    out.extend_from_slice(&0xdeadbeefu32.to_le_bytes());
    println!("{:?} {}", out, le32(&out));
    let f = u16::from_le_bytes;
    let v: Vec<u16> = [[1u8, 0], [0, 1]].into_iter().map(u16::from_be_bytes).collect();
    println!("{} {:?}", f([1, 2]), v);
    const C: [u8; 4] = 0x01020304u32.to_be_bytes();
    println!("{:?}", C);
}
