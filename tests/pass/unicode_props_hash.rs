// Every Unicode scalar (all 0x110000 code points) through the Unicode
// char predicates, to_digit/is_digit, the trim family, and the Debug
// forms of char and str, folded into FNV hashes difftest compares with
// rustc. from_digit runs over its whole non-panic domain. The predicate
// tables are generated from rustc itself (tools/gen_unicode.rs).

fn mix(h: u64, x: u64) -> u64 {
    (h ^ x).wrapping_mul(0x00000100000001b3)
}

fn mix_str(h: u64, s: &str) -> u64 {
    let mut h = mix(h, s.len() as u64);
    for c in s.chars() {
        h = mix(h, c as u64);
    }
    h
}

fn main() {
    let mut hp: u64 = 0xcbf29ce484222325;
    let mut hd: u64 = 0xcbf29ce484222325;
    let mut ht: u64 = 0xcbf29ce484222325;
    let mut hc: u64 = 0xcbf29ce484222325;
    let mut hs: u64 = 0xcbf29ce484222325;
    let mut counts = [0u64; 7];
    let mut s = String::new();
    let radices = [2u32, 8, 10, 16, 36];
    let mut cp: u32 = 0;
    while cp <= 0x10FFFF {
        if let Some(c) = char::from_u32(cp) {
            let p = [
                c.is_alphabetic(),
                c.is_numeric(),
                c.is_alphanumeric(),
                c.is_lowercase(),
                c.is_uppercase(),
                c.is_whitespace(),
                c.is_control(),
            ];
            let mut bits: u64 = 0;
            let mut i = 0;
            while i < 7 {
                if p[i] {
                    bits |= 1 << i;
                    counts[i] += 1;
                }
                i += 1;
            }
            hp = mix(hp, bits ^ (cp as u64));
            let mut r = 0;
            while r < radices.len() {
                let radix = radices[r];
                let d = match c.to_digit(radix) {
                    Some(v) => v as u64 + 1,
                    None => 0,
                };
                hd = mix(hd, d ^ ((c.is_digit(radix) as u64) << 40) ^ (radix as u64));
                r += 1;
            }
            // Trim of c + 'x' + c sees c at both ends, and its content.
            s.clear();
            s.push(c);
            s.push('x');
            s.push(c);
            ht = mix_str(ht, s.trim());
            ht = mix_str(ht, s.trim_start());
            ht = mix_str(ht, s.trim_end());
            hc = mix_str(hc, &format!("{:?}", c));
            // Debug of 'a' + c + 'b' sees c between base chars too.
            s.clear();
            s.push('a');
            s.push(c);
            s.push('b');
            hs = mix_str(hs, &format!("{:?}", s));
        }
        cp += 1;
    }
    // from_digit over its whole non-panic domain (radix 37 panics).
    let mut hf: u64 = 0xcbf29ce484222325;
    let mut radix: u32 = 0;
    while radix <= 36 {
        let mut d: u32 = 0;
        while d < 40 {
            let v = match char::from_digit(d, radix) {
                Some(c) => c as u64,
                None => 0xFFFFFFFF,
            };
            hf = mix(hf, v ^ (d as u64) ^ ((radix as u64) << 32));
            d += 1;
        }
        radix += 1;
    }
    println!("pred {hp:016x} digit {hd:016x} trim {ht:016x} from {hf:016x}");
    println!("dbgchar {hc:016x} dbgstr {hs:016x}");
    println!("counts {} {} {} {} {} {} {}", counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6]);
}
