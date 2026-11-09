// `to_ascii_lowercase`/`to_ascii_uppercase`, `encode_utf16` on
// `str` and on `char`, and `String::from_utf16_lossy` (tf-str),
// with capacities and size hints as std computes them.
fn main() {
    println!("{} {}", "AbCéX".to_ascii_lowercase(), "AbCéX".to_ascii_uppercase());
    let s = "HELLO".to_ascii_lowercase();
    println!("{} {}", s, s.capacity());
    println!("{:?}", "a\u{1F600}b".encode_utf16().collect::<Vec<u16>>());
    println!("{:?}", "".encode_utf16().collect::<Vec<u16>>());
    let mut e = "a\u{1F600}b".encode_utf16();
    println!("{:?}", e.size_hint());
    println!("{:?} {:?}", e.next(), e.size_hint());
    println!("{:?} {:?}", e.next(), e.size_hint());
    println!("{:?} {:?}", e.next(), e.size_hint());
    println!("{:?} {:?}", e.next(), e.size_hint());
    println!("{:?}", "abcd".encode_utf16().size_hint());
    let mut buf = [0u16; 2];
    println!("{:?}", 'a'.encode_utf16(&mut buf));
    let mut buf = [0u16; 4];
    println!("{:?}", '\u{1F600}'.encode_utf16(&mut buf));
    println!("{:?}", buf);
    println!("{}", String::from_utf16_lossy(&[104, 105]));
    println!("{}", String::from_utf16_lossy(&[0xD800, 97, 0xDC00, 0xD83D, 0xDE00]));
    let s = String::from_utf16_lossy(&[97; 17]);
    println!("{} {}", s.len(), s.capacity());
    let s = String::from_utf16_lossy(&[0xE9; 9]);
    println!("{} {}", s.len(), s.capacity());
    let s = String::from_utf16_lossy(&[]);
    println!("{} {}", s.len(), s.capacity());
    let v: Vec<u16> = "héllo".encode_utf16().collect();
    println!("{}", String::from_utf16_lossy(&v));
}
