// panic: encode_utf16: need 2 bytes to encode U+1D11E but buffer has just 1
// A buffer short by one unit panics inside core; std's text says
// "bytes" (TRIAGE T8).
fn read<T>(v: T) -> T {
    v
}

fn main() {
    let mut w = [0u16; 1];
    println!("{}", read('𝄞').encode_utf16(&mut w).len());
    println!("unreachable");
}
