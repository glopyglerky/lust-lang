// Discriminants whose written type is the discriminant type:
// a matching suffix, a typed const, arithmetic over it, a cast,
// a block with a typed let, and a primitive associated const.
const K: isize = 4;

#[repr(u8)]
enum Small {
    A = 1u8,
    B = 2 + 3,
}

enum E {
    A = K,
    B = K + 2,
    C = 1i32 as isize,
    D = { let x: isize = 9; x + 1 },
    E = if true { 20 } else { 21 },
    F = 1 << 5u32,
}

#[repr(i64)]
enum Big {
    Top = i64::MAX - 1,
}

fn main() {
    println!("{} {}", Small::A as i32, Small::B as i32);
    println!("{} {} {} {} {} {}", E::A as i32, E::B as i32, E::C as i32, E::D as i32, E::E as i32, E::F as i32);
    println!("{}", (Big::Top as i64) + 1 == i64::MAX);
}
