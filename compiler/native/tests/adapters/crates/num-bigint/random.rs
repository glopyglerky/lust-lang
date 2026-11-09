// num-bigint 0.4.6 on seeded random inputs: decimal and hex strings of
// up to 400 digits parse, print, and convert to f64 (round-to-odd on
// long magnitudes) as the real crate does; pow on random bases.
use num_bigint::BigInt;
use num_traits::ToPrimitive;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn main() {
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    let mut sum: u64 = 0;
    for i in 0..1500u64 {
        let radix = if i % 3 == 0 { 16 } else { 10 };
        let len = 1 + (rng.next() % 400) as usize;
        let mut s = String::new();
        if rng.next() % 2 == 0 {
            s.push('-');
        }
        for _ in 0..len {
            let d = (rng.next() % radix) as u8;
            s.push((if d < 10 { b'0' + d } else { b'a' + d - 10 }) as char);
        }
        let b = BigInt::parse_bytes(s.as_bytes(), radix as u32).unwrap();
        let text = b.to_string();
        let f = b.to_f64().unwrap();
        for c in text.chars() {
            sum = sum.wrapping_mul(131).wrapping_add(c as u64);
        }
        sum = sum.wrapping_mul(131).wrapping_add(f.to_bits());
        if i % 300 == 0 {
            println!("{} {:e} {:?}", text, f, b.to_i64());
        }
    }
    for _ in 0..200 {
        let base = (rng.next() % 2000) as i64 - 1000;
        let e = (rng.next() % 60) as u32;
        let p = BigInt::from(base).pow(e);
        sum = sum.wrapping_mul(131).wrapping_add(p.to_f64().unwrap().to_bits());
        sum = sum.wrapping_mul(131).wrapping_add(p.bits());
    }
    println!("{}", sum);
}
