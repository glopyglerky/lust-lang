const A: Option<u8> = 200u8.checked_add(100);
const B: Option<u8> = 3u8.checked_sub(5);
const C: Option<u8> = 20u8.checked_mul(10);
const D: Option<i8> = i8::MIN.checked_neg();
const E: u8 = 200u8.saturating_add(100);
const F: u8 = 3u8.saturating_sub(5);
const G: u8 = 100u8.saturating_mul(3);
const H: u8 = 200u8.wrapping_add(100);
const I: u8 = 3u8.wrapping_sub(5);
const J: i8 = (-100i8).wrapping_mul(2);
const K: i32 = (-7i32).abs();
const L: u32 = 2u32.pow(10);
const M: Option<u8> = 2u8.checked_pow(8);
const N: u8 = 2u8.saturating_pow(9);
const O: u32 = 0b1011u32.count_ones();
const P: u32 = 0u32.leading_zeros();
const Q: u32 = 16u32.trailing_zeros();
const R: u8 = 0b1000_0001u8.reverse_bits();
const S: u16 = 0x1234u16.swap_bytes();
const T: u8 = 0b1000_0001u8.rotate_left(1);
const U: u8 = 0b1000_0001u8.rotate_right(2);
const V: [u8; 4] = 0x01020304u32.to_be_bytes();
const W: u32 = u32::from_le_bytes([4, 3, 2, 1]);
const X: i32 = (-7i32).div_euclid(2);
const Y: i32 = (-7i32).rem_euclid(2);
const Z: u32 = 17u32.isqrt();
const MID: u32 = 3u32.midpoint(8);
const DIFF: u32 = 3u32.abs_diff(10);
const UABS: u32 = (-7i32).unsigned_abs();
const SIG: i32 = (-5i32).signum();
const POS: bool = 5i32.is_positive();
const NEG: bool = 5i32.is_negative();
const NP2: u32 = 17u32.next_power_of_two();
const LOG: u32 = 17u32.ilog2();
const SHL: Option<u8> = 1u8.checked_shl(8);
const WSHL: u8 = 1u8.wrapping_shl(9);
const MAXV: u8 = u8::MAX;
const MINV: i32 = i32::MIN;
const BITS: u64 = 1.5f64.to_bits();
const FABS: f64 = (-1.5f64).abs();
const FMAX: f64 = 1.5f64.max(2.5);
const FMIN: f32 = 1.5f32.min(2.5);
const SOME_OK: bool = A.is_none() && C.is_some();
const UNWRAPPED: u8 = C.unwrap();
const BIG: u128 = u128::MAX.wrapping_add(1);
const DIV0: Option<u8> = 3u8.checked_div(0);

const fn bump(x: u8) -> u8 {
    x.saturating_add(1)
}

const BUMPED: u8 = bump(255);

fn main() {
    println!("{:?} {:?} {:?} {:?}", A, B, C, D);
    println!("{} {} {} {} {} {}", E, F, G, H, I, J);
    println!("{} {} {:?} {}", K, L, M, N);
    println!("{} {} {} {:08b} {:x} {:08b} {:08b}", O, P, Q, R, S, T, U);
    println!("{:?} {:x} {} {} {}", V, W, X, Y, Z);
    println!("{} {} {} {} {} {} {}", MID, DIFF, UABS, SIG, POS, NEG, NP2);
    println!("{} {:?} {} {} {}", LOG, SHL, WSHL, MAXV, MINV);
    println!("{:x} {} {} {}", BITS, FABS, FMAX, FMIN);
    println!("{} {} {} {} {:?}", SOME_OK, UNWRAPPED, BUMPED, BIG, DIV0);
}
