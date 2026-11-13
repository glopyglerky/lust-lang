// expect: L0612
// `EncodeUtf16` is not a `DoubleEndedIterator` (tf-str).
fn main() {
    let mut it = "ab".encode_utf16();
    println!("{:?}", it.next_back());
}
