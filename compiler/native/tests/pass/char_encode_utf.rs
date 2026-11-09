// `char::encode_utf8` and `char::encode_utf16` (TRIAGE T8): the
// written prefix as `&mut str` / `&mut [u16]`, buffers bigger than
// the encoding, and exact-size buffers. Short buffers panic in
// the panic cases below.
fn read<T>(v: T) -> T {
    v
}

fn main() {
    let mut b = [0u8; 4];
    let s = read('é').encode_utf8(&mut b);
    println!("{} {}", s.len(), s);
    let mut b2 = [0u8; 8];
    let n = read('𝄞').encode_utf8(&mut b2).len();
    println!("{} {:?}", n, &b2[..n]);
    let mut b3 = [0u8; 1];
    println!("{}", read('a').encode_utf8(&mut b3));
    let mut b4 = [7u8; 3];
    {
        let s4 = read('€').encode_utf8(&mut b4);
        println!("{} {}", s4.len(), s4);
    }
    println!("{:?}", b4);
    let mut w = [0u16; 2];
    let m = read('𝄞').encode_utf16(&mut w).len();
    println!("{} {:?}", m, &w[..m]);
    let mut w2 = [0u16; 4];
    let s2 = read('é').encode_utf16(&mut w2);
    println!("{:?}", s2);
    let mut w3 = [0u16; 1];
    println!("{:?}", read('Z').encode_utf16(&mut w3));
    let mut v: Vec<u16> = vec![0; 2];
    let got = read('𐍈').encode_utf16(&mut v[..]).len();
    println!("{} {:?}", got, v);
}
