// `repr` combinations rustc accepts: packed with C in either
// order, duplicate aligns take the largest, a transparent enum of
// one variant, and a transparent struct whose other fields are
// zero-sized.
#[repr(packed, C)]
struct P1 {
    a: u8,
    b: u32,
}
#[repr(C, packed)]
struct P2 {
    a: u8,
    b: u32,
}
#[repr(align(8), align(16))]
struct A {
    x: u8,
}
#[repr(transparent)]
enum One {
    V(u32),
}
#[repr(transparent)]
struct Wrap(u32, ());
#[repr(Rust, packed)]
struct P3 {
    a: u8,
    b: u32,
}

fn main() {
    let one = One::V(41);
    let n = match one {
        One::V(v) => v,
    };
    println!(
        "{} {} {} {} {} {} {}",
        std::mem::size_of::<P1>(),
        std::mem::size_of::<P2>(),
        std::mem::align_of::<A>(),
        std::mem::size_of::<One>(),
        n + 1,
        Wrap(3, ()).0,
        std::mem::size_of::<P3>()
    );
}
