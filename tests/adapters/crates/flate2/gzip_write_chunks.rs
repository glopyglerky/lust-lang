// flate2 1.1.10 GzEncoder driven in chunks with flush() calls into a
// writer that takes at most 1000 bytes per call: the bytes and the
// order and size of every write and flush call match the real crate
// (its 32 KiB buffer, sync flush, and finish loops). Then the Drop
// path: an encoder dropped without finish writes the same trailer.
use std::io::{self, Write};

struct Sink {
    out: Vec<u8>,
    calls: Vec<usize>,
    flushes: u32,
}

impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let k = buf.len().min(1000);
        self.out.extend_from_slice(&buf[..k]);
        self.calls.push(k);
        Ok(k)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        Ok(())
    }
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
    let input: Vec<u8> = (0..100000u32).map(|i| (i * 7 % 251) as u8 ^ (i / 997) as u8).collect();
    let sink = Sink { out: Vec::new(), calls: Vec::new(), flushes: 0 };
    let mut e = flate2::write::GzEncoder::new(sink, flate2::Compression::new(6));
    for (i, c) in input.chunks(7777).enumerate() {
        let n = e.write(c).unwrap();
        e.write_all(&c[n..]).unwrap();
        if i % 3 == 2 {
            e.flush().unwrap();
            println!("after flush {}: {} bytes out", i, e.get_ref().out.len());
        }
    }
    let s = e.finish().unwrap();
    println!("len {} fnv {:016x} calls {} flushes {}", s.out.len(), fnv(&s.out), s.calls.len(), s.flushes);
    println!("first calls {:?}", &s.calls[..8]);

    let mut dropped = Vec::new();
    {
        let mut e = flate2::write::GzEncoder::new(&mut dropped, flate2::Compression::best());
        e.write_all(&input[..5000]).unwrap();
    }
    println!("dropped len {} fnv {:016x}", dropped.len(), fnv(&dropped));
}
