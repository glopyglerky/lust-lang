use std::num::NonZeroU64;

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct Large(pub [u64; 6]);

#[inline(never)]
pub fn large(value: Large) -> Large {
    let mut result = value;
    result.0[5] += 1;
    result
}

#[inline(never)]
pub fn slice(value: &[u64]) -> &[u64] {
    &value[1..]
}

#[inline(never)]
pub fn niche(value: Option<NonZeroU64>) -> u64 {
    value.map_or(0, NonZeroU64::get)
}

#[inline(never)]
pub fn identity<T>(value: T) -> T {
    value
}

#[inline(never)]
pub fn invoke(f: &mut dyn FnMut() -> u64) -> u64 {
    f()
}
