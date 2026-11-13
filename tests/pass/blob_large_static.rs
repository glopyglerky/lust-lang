// A 150 KB include_bytes! blob in a static and in const fns: one
// buffer end to end, so const hashes over it stay linear and in
// step bound (the old per-element value hit L0630; fx-blob).
static B: &[u8] = include_bytes!("blob_large_data.bin");
const A: &[u8; 150000] = include_bytes!("blob_large_data.bin");
const fn fnv(b: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    let mut i = 0usize;
    while i < b.len() {
        h = (h ^ b[i] as u64).wrapping_mul(0x100000001b3);
        i += 1;
    }
    h
}
const fn xorshift(b: &[u8]) -> u64 {
    let mut h = 0x9e3779b97f4a7c15u64;
    let mut i = 0usize;
    while i < b.len() {
        h ^= b[i] as u64;
        h ^= h << 13;
        h ^= h >> 7;
        h ^= h << 17;
        i += 1;
    }
    h
}
const fn chunked(b: &[u8]) -> u64 {
    // Eight bytes per step, by index (the while-let tail form of
    // this loop is a known borrowck false reject today, L0306).
    let mut h = b.len() as u64;
    let mut i = 0usize;
    while i + 8 <= b.len() {
        h ^= b[i] as u64
            | (b[i + 1] as u64) << 8
            | (b[i + 2] as u64) << 16
            | (b[i + 3] as u64) << 24
            | (b[i + 4] as u64) << 32
            | (b[i + 5] as u64) << 40
            | (b[i + 6] as u64) << 48
            | (b[i + 7] as u64) << 56;
        h = h.wrapping_mul(0x100000001b3);
        i += 8;
    }
    while i < b.len() {
        h = (h ^ b[i] as u64).wrapping_mul(16777619);
        i += 1;
    }
    h
}
const fn subtail(b: &[u8]) -> u64 {
    // Pattern slicing is the const-stable slicing form.
    match b {
        [_, _, _, _, _, _, _, rest @ ..] => chunked(rest),
        _ => 0,
    }
}
const H1: u64 = fnv(include_bytes!("blob_large_data.bin"));
const H2: u64 = xorshift(B);
const H3: u64 = subtail(B);
const N: usize = include_bytes!("blob_large_data.bin").len();
const MID: u8 = B[75000];
const SPLIT0: u8 = B.split_at(9).1[0];
const TWEAK: u8 = {
    // A write to one copy of a byte array leaves the other alone.
    let mut a = *b"abcdef";
    let copy = a;
    a[0] = b'z';
    copy[0]
};
fn main() {
    println!("{} {} {} {} {}", B.len(), A.len(), N, MID, SPLIT0);
    println!("{} {} {}", H1, H2, H3);
    println!("{} {} {}", fnv(B) == H1, xorshift(A) == H2, subtail(B) == H3);
    println!("{:?} {:?}", &B[10..20], &B[..8]);
    println!("{} {} {}", B[0], B[B.len() - 1], B.iter().map(|&x| x as u64).sum::<u64>());
    let local = include_bytes!("blob_large_data.bin");
    println!("{} {} {}", fnv(local), TWEAK, b"abc"[1]);
}
