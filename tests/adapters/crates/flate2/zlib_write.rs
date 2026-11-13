// flate2 1.1.10 write::ZlibEncoder: the bytes, the order and size of the
// inner writer's write and flush calls, total_in/total_out, flush_finish,
// try_finish, and the Drop path match the real crate, at every level.
// ts-rust's layergraph.rs encodes mermaid URLs with
// ZlibEncoder::new(Vec::new(), Compression::best()), write_all, finish.
use std::io::{self, Write};

struct Sink {
    out: Vec<u8>,
    calls: Vec<usize>,
    flushes: u32,
}

impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let k = buf.len().min(1500);
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
    let text = b"graph TD\n  A[Layer one] --> B[Layer two]\n  B --> C{decide}\n  C -->|yes| D[done]\n  C -->|no| A\n";
    let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    e.write_all(text).unwrap();
    let z = e.finish().unwrap();
    println!("mermaid {} {:02x?}", z.len(), &z[..]);

    let input: Vec<u8> = (0..120000u32).map(|i| (i * 13 % 241) as u8 ^ (i / 1013) as u8).collect();
    for level in 0..=9 {
        let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(level));
        e.write_all(&input).unwrap();
        println!("level {level}: in {} out {}", e.total_in(), e.total_out());
        let v = e.finish().unwrap();
        println!("level {level}: len {} fnv {:016x}", v.len(), fnv(&v));
    }

    let sink = Sink { out: Vec::new(), calls: Vec::new(), flushes: 0 };
    let mut e = flate2::write::ZlibEncoder::new(sink, flate2::Compression::default());
    for (i, c) in input.chunks(9001).enumerate() {
        let n = e.write(c).unwrap();
        e.write_all(&c[n..]).unwrap();
        if i % 4 == 3 {
            e.flush().unwrap();
            println!("after flush {}: {} bytes out", i, e.get_ref().out.len());
        }
    }
    e.try_finish().unwrap();
    e.try_finish().unwrap();
    let s = e.finish().unwrap();
    println!("len {} fnv {:016x} calls {} flushes {}", s.out.len(), fnv(&s.out), s.calls.len(), s.flushes);
    println!("first calls {:?}", &s.calls[..6]);

    let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    e.write_all(&input[..20000]).unwrap();
    let v = e.flush_finish().unwrap();
    println!("flush_finish len {} fnv {:016x}", v.len(), fnv(&v));

    let mut dropped = Vec::new();
    {
        let mut e = flate2::write::ZlibEncoder::new(&mut dropped, flate2::Compression::best());
        e.write_all(&input[..7000]).unwrap();
        e.get_mut().len();
    }
    println!("dropped len {} fnv {:016x}", dropped.len(), fnv(&dropped));

    let empty = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::none()).finish().unwrap();
    println!("empty {:02x?}", empty);
}
