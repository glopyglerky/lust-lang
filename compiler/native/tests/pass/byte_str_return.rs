// A byte string literal is `&'static [u8; N]`: returning a borrow
// of it (as a slice, as an array reference, or a subslice) is sound.
// The literal's temporary gets static storage in the generated C,
// so the borrow stays valid after the frame is gone.
fn as_slice() -> &'static [u8] {
    b"ab"
}

fn as_array() -> &'static [u8; 3] {
    b"xyz"
}

fn sub() -> &'static [u8] {
    &b"abcd"[1..3]
}

struct Wrap {
    s: &'static [u8],
}

fn in_struct() -> Wrap {
    Wrap { s: b"in" }
}

fn main() {
    println!("{:?} {:?} {:?}", as_slice(), as_array(), sub());
    println!("{:?} {:?}", in_struct().s, b"direct");
    let mut copy = *b"ab";
    copy[0] = 122;
    println!("{:?} {:?}", copy, as_slice());
}
