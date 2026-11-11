#[repr(C)]
struct Pair {
    first: u32,
    second: u32,
}

fn main() {
    let offset = std::mem::offset_of!(Pair, second);
    assert_eq!(offset, 4);
    println!("{}", offset);
}
