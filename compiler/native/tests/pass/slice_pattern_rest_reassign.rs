// A slice pattern's `rest @ ..` binding assigned back to the scrutinee
// variable (ts-rust frontend/bundled.rs const_hash): the rest borrows the
// base slice, not the pattern's subslice temporary, so the loop and the
// const evaluation work as in rustc.
const fn mix(h: u64) -> u64 {
    h.wrapping_mul(0x9E3779B97F4A7C15) ^ (h >> 29)
}

pub const fn const_hash(bytes: &[u8]) -> u64 {
    let mut hash = bytes.len() as u64;
    let mut rest = bytes;
    while let [b0, b1, b2, b3, tail @ ..] = rest {
        hash = mix(hash ^ u32::from_le_bytes([*b0, *b1, *b2, *b3]) as u64);
        rest = tail;
    }
    while let [byte, tail @ ..] = rest {
        hash ^= *byte as u64;
        rest = tail;
    }
    mix(hash)
}

const H: u64 = const_hash(b"hello, lust");

fn parts(mut rest: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    loop {
        match rest {
            [_, tail @ ..] => {
                out.push(tail);
                rest = tail;
            }
            [] => break,
        }
    }
    out
}

fn main() {
    println!("{} {}", H, const_hash(b"abc"));
    println!("{:?}", parts(b"xyz"));
    let mut v = vec![1u8, 2, 3, 4];
    if let [first, mid @ .., _] = &mut v[..] {
        *first = 9;
        mid.reverse();
    }
    println!("{:?}", v);
}
