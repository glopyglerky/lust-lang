// `make_ascii_lowercase` and `make_ascii_uppercase` on `u8` and
// `char`: in place through a `&mut` receiver, no-op outside the
// ASCII letter ranges (TRIAGE T8).
struct Holder {
    b: u8,
    c: char,
}

fn main() {
    let mut b = b'A';
    b.make_ascii_lowercase();
    println!("{}", b as char);
    b.make_ascii_uppercase();
    println!("{}", b as char);
    let mut d = b'5';
    d.make_ascii_lowercase();
    d.make_ascii_uppercase();
    println!("{}", d);
    let mut c = 'Z';
    c.make_ascii_lowercase();
    println!("{c}");
    c.make_ascii_uppercase();
    println!("{c}");
    let mut h = Holder { b: b'q', c: 'É' };
    h.b.make_ascii_uppercase();
    h.c.make_ascii_lowercase();
    println!("{} {}", h.b as char, h.c);
    let mut arr = [b'a', b'B', b'!'];
    arr[0].make_ascii_uppercase();
    arr[1].make_ascii_lowercase();
    arr[2].make_ascii_lowercase();
    println!("{:?}", arr);
}
