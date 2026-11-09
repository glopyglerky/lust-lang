// `[u8]::utf8_chunks` (tf-slice): valid/invalid splits as core's
// scan gives them, including truncated and overlong sequences.
fn show(b: &[u8]) {
    for chunk in b.utf8_chunks() {
        print!("[{:?}|{:?}]", chunk.valid(), chunk.invalid());
    }
    println!();
}
fn main() {
    show(b"hello");
    show(b"");
    show(b"a\xffb");
    show(b"\xff\xfe");
    show("héllo wörld".as_bytes());
    show(b"foo\xf1\x80bar");
    show(b"\xc3");
    show(b"\xc3\x28");
    show(b"\xe0\x80\x80");
    show(b"\xed\xa0\x80");
    show(b"\xf0\x9f\xa6\x80x");
    show(b"\xf4\x90\x80\x80");
    show(b"\xe1\x80");
    show(b"a\xc2b");
    let first = b"ab\xff".utf8_chunks().next().unwrap();
    println!("{:?} {}", first.valid(), first.invalid().len());
    let n = b"a\xffb\xfec".utf8_chunks().map(|c| c.valid().len() + c.invalid().len()).sum::<usize>();
    println!("{n}");
    let v: Vec<u8> = vec![0x61, 0xFF, 0x62];
    show(&v);
}
