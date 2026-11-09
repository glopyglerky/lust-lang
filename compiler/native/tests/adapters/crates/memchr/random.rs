// memchr 2.8.2 on seeded random haystacks over a small alphabet, so
// matches are frequent: every search fn, every iterator, folded into
// per-fn checksums.
use memchr::memmem;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn bytes(&mut self, max: u64) -> Vec<u8> {
        let len = self.below(max + 1);
        let mut v = Vec::new();
        for _ in 0..len {
            v.push(b'a' + self.below(4) as u8);
        }
        v
    }
}

fn fold(sum: u64, r: Option<usize>) -> u64 {
    let x = match r {
        Some(i) => i as u64 + 1,
        None => 0,
    };
    sum.wrapping_mul(31).wrapping_add(x)
}

fn main() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut sums = [0u64; 12];
    for _ in 0..3000 {
        let h = rng.bytes(80);
        let n = rng.bytes(4);
        let b1 = b'a' + rng.below(5) as u8;
        let b2 = b'a' + rng.below(5) as u8;
        let b3 = b'a' + rng.below(5) as u8;
        sums[0] = fold(sums[0], memchr::memchr(b1, &h));
        sums[1] = fold(sums[1], memchr::memrchr(b1, &h));
        sums[2] = fold(sums[2], memchr::memchr2(b1, b2, &h));
        sums[3] = fold(sums[3], memchr::memrchr2(b1, b2, &h));
        sums[4] = fold(sums[4], memchr::memchr3(b1, b2, b3, &h));
        sums[5] = fold(sums[5], memchr::memrchr3(b1, b2, b3, &h));
        sums[6] = fold(sums[6], memmem::find(&h, &n));
        sums[7] = fold(sums[7], memmem::rfind(&h, &n));
        for i in memchr::memchr3_iter(b1, b2, b3, &h) {
            sums[8] = fold(sums[8], Some(i));
        }
        for i in memmem::find_iter(&h, &n) {
            sums[9] = fold(sums[9], Some(i));
        }
        for i in memmem::rfind_iter(&h, &n) {
            sums[10] = fold(sums[10], Some(i));
        }
        let f = memmem::Finder::new(&n);
        sums[11] = fold(sums[11], f.find(&h));
    }
    for s in sums {
        println!("{}", s);
    }
}
