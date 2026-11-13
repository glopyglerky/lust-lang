// flate2 1.1.10 GzEncoder over a writer that takes at most 4096 bytes
// per call and fails once (an error, or Ok(0) for WriteZero): the
// error comes back unchanged, and when it stops finish between two
// deflate Finish runs, a later flush() reports upstream's
// CompressError (Display, Debug, {:#?}, and padding as std prints an
// io::Error with that payload).
use std::io::{self, Write};

struct Flaky {
    out: Vec<u8>,
    n: usize,
    fail_at: usize,
    zero: bool,
}

impl Write for Flaky {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.n += 1;
        if self.n == self.fail_at {
            return if self.zero { Ok(0) } else { Err(io::Error::new(io::ErrorKind::Other, "disk full")) };
        }
        let k = buf.len().min(4096);
        self.out.extend_from_slice(&buf[..k]);
        Ok(k)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn run(input: &[u8], fail_at: usize, zero: bool) {
    let w = Flaky { out: Vec::new(), n: 0, fail_at, zero };
    let mut e = flate2::write::GzEncoder::new(w, flate2::Compression::new(6));
    println!("write_all {:?}", e.write_all(input).map_err(|x| x.kind()));
    let r = e.try_finish();
    println!("try_finish {:?}", r);
    if r.is_err() {
        match e.flush() {
            Ok(()) => println!("flush ok"),
            Err(x) => println!("flush: {} | {:?} | {:#?} | [{:>30}] | {:?}", x, x, x, x, x.kind()),
        }
        println!("try_finish again {:?}", e.try_finish().map_err(|x| x.to_string()));
    }
    println!("out {} bytes", e.get_ref().out.len());
}

fn main() {
    let mut s: u64 = 99;
    let input: Vec<u8> = (0..300000)
        .map(|i| {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((s >> 33) % 3) as u8 + (i as u8 & 1)
        })
        .collect();
    for (fail_at, zero) in [(1, false), (3, true), (12, false), (16, true), (20, false)] {
        println!("-- fail at write {} (zero {})", fail_at, zero);
        run(&input, fail_at, zero);
    }
}
