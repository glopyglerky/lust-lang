// panic: encode_utf8: need 2 bytes to encode U+00E9 but buffer has just 1
// A buffer short by one byte panics inside core (TRIAGE T8).
fn read<T>(v: T) -> T {
    v
}

fn main() {
    let mut b = [0u8; 1];
    println!("{}", read('é').encode_utf8(&mut b).len());
    println!("unreachable");
}
