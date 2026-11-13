// str::to_uppercase/to_lowercase over every char (in chunks), the final
// sigma contexts, and the result capacities, against rustc (W-STD-B).
fn digest(s: &str) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for b in s.bytes() {
        h = (h ^ b as u64).wrapping_mul(1099511628211);
    }
    h
}

fn push_utf8(v: &mut Vec<u8>, u: u32) {
    if u < 0x80 {
        v.push(u as u8);
    } else if u < 0x800 {
        v.push(0xC0 | (u >> 6) as u8);
        v.push(0x80 | (u & 0x3F) as u8);
    } else if u < 0x10000 {
        v.push(0xE0 | (u >> 12) as u8);
        v.push(0x80 | ((u >> 6) & 0x3F) as u8);
        v.push(0x80 | (u & 0x3F) as u8);
    } else {
        v.push(0xF0 | (u >> 18) as u8);
        v.push(0x80 | ((u >> 12) & 0x3F) as u8);
        v.push(0x80 | ((u >> 6) & 0x3F) as u8);
        v.push(0x80 | (u & 0x3F) as u8);
    }
}

fn main() {
    let mut bytes: Vec<u8> = Vec::new();
    let mut n = 0;
    let mut all_up = 0u64;
    let mut all_lo = 0u64;
    for u in 0u32..0x20000 {
        if u >= 0xD800 && u < 0xE000 {
            continue;
        }
        push_utf8(&mut bytes, u);
        n += 1;
        if n == 256 {
            let chunk = String::from_utf8(bytes.clone()).unwrap();
            let up = chunk.to_uppercase();
            let lo = chunk.to_lowercase();
            all_up = all_up.wrapping_mul(31).wrapping_add(digest(&up) ^ up.len() as u64 ^ (up.capacity() as u64) << 32);
            all_lo = all_lo.wrapping_mul(31).wrapping_add(digest(&lo) ^ lo.len() as u64 ^ (lo.capacity() as u64) << 32);
            bytes.clear();
            n = 0;
        }
    }
    println!("{all_up:x} {all_lo:x}");
    for s in ["Σ", "ΑΣ", "ΑΣ ", "ΣΑ", "ΑΣΑ", "Α'Σ", "Α'Σ'", "ΑΣ.", "ΣΣ", "ΑΣΣ", "a\u{301}Σ", "ΑΣ\u{301}", "1Σ", "ΑΣ1", "ΑΣ'b", "ὈΔΥΣΣΕΎΣ", "hello WORLD Σ", "ǅǄǆ", "ﬃ İ ŉ ΐ ẞ", "ABC déf"] {
        let lo = s.to_lowercase();
        let up = s.to_uppercase();
        println!("{s:?} -> {lo:?} {} {} / {up:?} {} {}", lo.len(), lo.capacity(), up.len(), up.capacity());
    }
    let long = "ß".repeat(20) + &"a".repeat(30);
    let u = long.to_uppercase();
    println!("{} {} {}", u.len(), u.capacity(), &u[..10]);
}
