// `copy_from_slice` is one memcpy of the bytes (T is Copy, the slices cannot overlap), not an
// element loop: a key built by fixed-size puts and read back as words (the case whose byte
// stores stalled), 2, 8, 12 and 8-byte float elements, a struct with padding, zero-sized
// elements, empty slices, and run-time lengths at odd offsets.
#[derive(Clone, Copy, Debug, PartialEq)]
struct P {
    a: u8,
    b: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Z;

struct Key {
    len: usize,
    buf: [u8; 24],
}

impl Key {
    fn put<const N: usize>(&mut self, bytes: [u8; N]) {
        let len = self.len;
        self.buf[len..len + N].copy_from_slice(&bytes);
        self.len = len + N;
    }
}

fn main() {
    // 3 under the test runner: lengths below are not constants.
    let n = std::env::args().count();

    let mut k = Key { len: 0, buf: [0; 24] };
    k.put(0x0102_0304_0506_0708u64.to_le_bytes());
    k.put((n as u32).to_le_bytes());
    k.put([0xaa_u8]);
    k.put(0xbeef_u16.to_le_bytes());
    let w0 = u64::from_le_bytes(k.buf[0..8].try_into().unwrap());
    let w1 = u64::from_le_bytes(k.buf[8..16].try_into().unwrap());
    println!("{} {:#x} {:#x}", k.len, w0, w1);

    let mut a16 = [0u16; 5];
    a16[1..4].copy_from_slice(&[7, 8, 9]);
    println!("{:?}", a16);
    let mut a64 = vec![0u64; n + 2];
    let src: Vec<u64> = (0..(n as u64 + 2)).map(|i| i * 0x1_0000_0001).collect();
    a64.copy_from_slice(&src);
    println!("{:?}", a64);
    let mut t3 = [(0u32, 0u32, 0u32); 3];
    t3[..2].copy_from_slice(&[(1, 2, 3), (4, 5, 6)]);
    println!("{:?}", t3);
    let mut f = [0.0f64; 3];
    f.copy_from_slice(&[1.5, -2.25, 1e300]);
    println!("{:?}", f);

    let mut ps = [P { a: 0, b: 0 }; 3];
    ps[1..].copy_from_slice(&[P { a: 1, b: 2 }, P { a: 3, b: 4 }]);
    println!("{:?}", ps);

    let mut z = [Z; 4];
    z.copy_from_slice(&[Z; 4]);
    println!("{:?}", z);
    let mut u: Vec<()> = vec![(); n];
    let us = vec![(); n];
    u.copy_from_slice(&us);
    println!("{}", u.len());
    let mut e: Vec<u32> = Vec::new();
    e.copy_from_slice(&[]);
    let mut e2 = [0u8; 4];
    e2[n..n].copy_from_slice(&[]);
    println!("{:?} {:?}", e, e2);

    let mut buf = vec![0u8; 32];
    let text = b"the quick brown fox jumps over";
    let m = n + 10;
    buf[n..n + m].copy_from_slice(&text[1..1 + m]);
    println!("{:?}", std::str::from_utf8(&buf[n..n + m]).unwrap());
    let mut words = vec![0u32; 2 * n];
    let from: Vec<u32> = (0..(4 * n as u32)).collect();
    words[1..2 * n - 1].copy_from_slice(&from[n + 1..3 * n - 1]);
    println!("{:?}", words);
}
