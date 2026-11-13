// A `repr(C)` enum's tag is a C `int` when every discriminant
// fits in it, a C `unsigned int` when they all fit there, and an
// 8-byte signed tag otherwise.
#[repr(C)]
enum E {
    A = -1,
    B = 4294967295,
}
#[repr(C)]
enum F {
    A = 2147483648,
}
#[repr(C)]
enum H {
    A = 4294967295,
}
#[repr(C)]
enum J {
    A = -2147483649,
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
enum N {
    A = 1,
    B = 3000000000,
}
#[repr(C)]
enum W {
    A = 1 << 31,
    B,
}
fn main() {
    println!("{} {} {}", E::B as u64, E::A as i64, std::mem::size_of::<E>());
    println!("{} {}", F::A as u64, F::A as i64);
    println!("{} {} {}", H::A as u64, H::A as i32, std::mem::size_of::<H>());
    println!("{} {}", J::A as i64, std::mem::size_of::<J>());
    let n = N::B;
    println!("{} {}", n as u32, n as i64);
    println!("{} {}", W::A as i64, W::B as i64);
    let _ = N::A;
}
