// panic: copy_from_slice: source slice length (2) does not match destination slice length (4)
// `copy_from_slice` with a short source panics at the call with
// core's exact text (tf-slice).
fn main() {
    let n = std::env::args().count();
    let mut a = [0u8; 4];
    println!("start {n}");
    a.copy_from_slice(b"ab");
    println!("{:?}", a);
}
