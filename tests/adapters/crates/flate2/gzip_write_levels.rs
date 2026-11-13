// flate2 1.1.10 GzEncoder at every level 0..=10 (10 is miniz's
// "uber" level; the header's XFL byte follows the level) over text,
// random, and run-heavy inputs that cross the 32 KiB window, so the
// stored, fixed, and dynamic block paths and the lazy and greedy
// parsers all run. Prints the length and an FNV-64 of each stream.
use std::io::Write;

fn data(kind: u32, len: usize) -> Vec<u8> {
    let mut s: u64 = 12345;
    let mut v = Vec::with_capacity(len);
    let words: [&[u8]; 6] = [b"alpha ", b"beta ", b"gamma ", b"delta\n", b"eps ", b"zeta "];
    while v.len() < len {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let r = s >> 33;
        match kind {
            0 => v.extend_from_slice(words[(r % 6) as usize]),
            1 => v.push(r as u8),
            _ => {
                for _ in 0..(r % 200) {
                    v.push((r >> 8) as u8 % 3);
                }
            }
        }
    }
    v.truncate(len);
    v
}

fn fnv(b: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &x in b {
        h ^= x as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn main() {
    for kind in 0..3 {
        let input = data(kind, 70000);
        for level in 0..=10u32 {
            let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(level));
            e.write_all(&input).unwrap();
            let v = e.finish().unwrap();
            println!("kind {} level {:2}: len {:6} xfl {} fnv {:016x}", kind, level, v.len(), v[8], fnv(&v));
        }
    }
}
