// str::replace leaves the capacities Rust leaves: exact len for the
// one-byte to one-byte path, else a start at the source length when
// the pattern is no longer than the replacement (0 otherwise) and
// amortized growth by pieces.
fn show(s: String) {
    println!("{:?} {}", s, s.capacity());
}
fn main() {
    show("abc".replace("b", "xyz"));
    show("aXbXc".replace("X", ""));
    show("aXbXc".replace("X", "Y"));
    show("aXbXc".replace("X", "YZ"));
    show("abc".replace("x", ""));
    show("abc".replace("x", "yy"));
    show("abc".replace("xy", "zw"));
    show("".replace("x", "y"));
    show("abc".replace("", "-"));
    show("abc".replace("", ""));
    show("héllo".replace("é", "ee"));
    show("abcabcabcabcabcabc".replace("bc", "XYZW"));
    show("aaaaaaaaaaaaaaaaa".replace("aa", "b"));
    show("hello world hello".replace("hello", "goodbye"));
    show("aaa".replace("aa", "b"));
    let big = "0123456789".repeat(10);
    show(big.replace("8", "8888"));
}
