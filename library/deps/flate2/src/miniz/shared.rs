#[doc(hidden)]
pub const MZ_ADLER32_INIT: u32 = 1;

#[doc(hidden)]
pub const MZ_DEFAULT_WINDOW_BITS: i32 = 15;

pub const HUFFMAN_LENGTH_ORDER: [u8; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// flate2 turns on miniz_oxide's `simd` feature (simd-adler32); the
/// checksum value is the same in every Adler-32 implementation.
pub fn update_adler32(adler: u32, data: &[u8]) -> u32 {
    crate::adler32(data, adler)
}
