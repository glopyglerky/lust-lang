// ryu-js 1.0.2 Buffer::format on seeded random f64 bit patterns (every
// exponent, subnormals, NaNs) and on random short decimals: byte-exact
// text, folded into a checksum with a sample printed in full.
fn main() {
    let mut st: u64 = 0x853C_49E6_748F_EA9B;
    let mut b = ryu_js::Buffer::new();
    let mut sum: u64 = 0;
    let mut total = 0usize;
    for i in 0..20000u64 {
        st ^= st << 13;
        st ^= st >> 7;
        st ^= st << 17;
        let v = if i % 2 == 0 { f64::from_bits(st) } else { ((st % 2000001) as f64 - 1000000.0) / 1000.0 };
        let s = b.format(v);
        total += s.len();
        for c in s.chars() {
            sum = sum.wrapping_mul(131).wrapping_add(c as u64);
        }
        if i % 1999 == 0 {
            println!("{}", s);
        }
    }
    println!("{} {}", total, sum);
}
