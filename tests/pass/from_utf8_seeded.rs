// UTF-8 validation over 20000 seeded byte strings near the edge cases.
fn next(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}
fn main() {
    let picks: [u8; 24] = [0x00, 0x41, 0x7f, 0x80, 0x8f, 0x90, 0x9f, 0xa0, 0xbf, 0xc0, 0xc1, 0xc2, 0xdf, 0xe0, 0xe1, 0xec, 0xed, 0xee, 0xef, 0xf0, 0xf1, 0xf3, 0xf4, 0xf5];
    let mut seed: u64 = 0x9e3779b97f4a7c15;
    let mut sum_valid: u64 = 0;
    let mut sum_len: u64 = 0;
    let mut oks: u64 = 0;
    let mut nones: u64 = 0;
    for round in 0..20000 {
        let n = (next(&mut seed) % 7) as usize;
        let mut v: Vec<u8> = Vec::new();
        for _ in 0..n {
            let r = next(&mut seed);
            if r % 4 == 0 {
                v.push((r >> 8) as u8);
            } else {
                v.push(picks[((r >> 8) % 24) as usize]);
            }
        }
        match String::from_utf8(v) {
            Ok(s) => {
                oks += 1;
                sum_len += s.len() as u64;
            }
            Err(e) => {
                let u = e.utf8_error();
                sum_valid += u.valid_up_to() as u64 * (round as u64 % 13 + 1);
                match u.error_len() {
                    Some(k) => sum_len += k as u64 * (round as u64 % 7 + 1),
                    None => nones += 1,
                }
                if round % 997 == 0 {
                    println!("{} {:?}", e, e.as_bytes());
                }
            }
        }
    }
    println!("{} {} {} {}", oks, nones, sum_valid, sum_len);
}
