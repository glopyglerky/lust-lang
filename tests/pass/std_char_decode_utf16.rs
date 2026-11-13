// char::decode_utf16 (library/core/src/char.rs, std's decode.rs): pairs, unpaired
// surrogates as errors (Display, Debug, unpaired_surrogate), size_hint, and use
// through builtin adapters (it steps by its own next, gx.iter_methods).
fn main() {
    let v: Vec<u16> = vec![0xD834, 0xDD1E, 0x006d, 0x0075, 0xDD1E, 0x0069, 0xD834, 0x63, 0xD800];
    let r: Vec<_> = char::decode_utf16(v.iter().copied()).map(|r| r.map_err(|e| e.unpaired_surrogate())).collect();
    println!("{:?}", r);
    let s: String = char::decode_utf16(v.iter().copied()).map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER)).collect();
    println!("{:?}", s);
    for x in char::decode_utf16(v.clone().into_iter()) {
        match x { Ok(c) => print!("{c}"), Err(e) => print!("[{e} {:?}]", e) }
    }
    println!();
    let mut it = char::decode_utf16([0xD800u16, 0x41, 0xDC00].into_iter());
    println!("{:?}", it.size_hint());
    it.next();
    println!("{:?} {:?}", it.size_hint(), it.count());
    let e = char::decode_utf16([0xDC01u16].into_iter()).next().unwrap().unwrap_err();
    println!("{}", e == e.clone());
}
