// `next_multiple_of`, `checked_next_multiple_of` (unsigned only) and
// `unsigned_abs` (signed to its unsigned twin), at runtime and in
// const items (TRIAGE T8). Panic cases live in the panic test below.
const CNM: u32 = 37u32.next_multiple_of(5);
const CCHK: Option<u8> = 250u8.checked_next_multiple_of(16);
const CCHK0: Option<u8> = 5u8.checked_next_multiple_of(0);
const CUA: u64 = (-9_000_000_000i64).unsigned_abs();
const CUA_MIN: u32 = i32::MIN.unsigned_abs();

fn read<T>(v: T) -> T {
    v
}

fn main() {
    println!("{} {:?} {:?} {} {}", CNM, CCHK, CCHK0, CUA, CUA_MIN);
    println!("{}", read(37usize).next_multiple_of(read(5usize)));
    println!("{}", read(40usize).next_multiple_of(5));
    println!("{}", read(0u64).next_multiple_of(7));
    println!("{}", read(1u16).next_multiple_of(1));
    println!("{:?}", read(255u8).checked_next_multiple_of(2));
    println!("{:?}", read(254u8).checked_next_multiple_of(2));
    println!("{:?}", read(u128::MAX - 3).checked_next_multiple_of(16));
    println!("{}", read(u128::MAX - 15).next_multiple_of(16) == u128::MAX - 15);
    println!("{} {} {}", read(-5i32).unsigned_abs(), read(5i32).unsigned_abs(), read(0i32).unsigned_abs());
    println!("{} {}", read(-128i8).unsigned_abs(), read(i128::MIN).unsigned_abs() == (1u128 << 127));
    println!("{}", read(-1isize).unsigned_abs());
    let n = read(8192usize);
    println!("{}", read(8193usize).next_multiple_of(n));
}
