// A user hasher's own `write_u32` is called; core's Hash sequences for
// char, tuples, Vec, arrays, and Option feed it (W-STD-B).
use std::hash::{Hash, Hasher};
struct F(u64);
impl Hasher for F {
    fn write(&mut self, b: &[u8]) {
        for x in b {
            self.0 = self.0.wrapping_mul(31).wrapping_add(*x as u64);
        }
    }
    fn write_u32(&mut self, i: u32) {
        println!("own write_u32 {i}");
        self.0 ^= i as u64;
    }
    fn finish(&self) -> u64 {
        self.0
    }
}
fn main() {
    let mut h = F(0);
    h.write_u32(7);
    'c'.hash(&mut h);
    (1u32, true).hash(&mut h);
    vec![1u16, 2].hash(&mut h);
    [3u8; 2].hash(&mut h);
    Some(-1i8).hash(&mut h);
    println!("{}", h.finish());
}
