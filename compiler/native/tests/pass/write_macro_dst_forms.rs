// write!/writeln! take their destination as one expression: `write!(&mut v, ..)`
// calls `(&mut v).write_fmt(..)`, so `.unwrap()` and `?` apply to the Result.
use std::fmt::Write as _;
use std::io::Write;

struct Counted(Vec<u8>, usize);

impl Write for Counted {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.1 += 1;
        self.0.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
    // One write per `write!`.
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.write_all(std::fmt::format(args).as_bytes())
    }
}

fn go(out: &mut Vec<u8>) -> std::io::Result<usize> {
    write!(&mut *out, "q{}", 1)?;
    writeln!(&mut *out, "!")?;
    Ok(out.len())
}

fn main() {
    let mut v: Vec<u8> = Vec::new();
    write!(&mut v, "a{}b{:>3}", 7, "x").unwrap();
    writeln!(&mut v, "-").expect("vec");
    let r = &mut v;
    write!(&mut *r, "[{}]", 2).unwrap();
    let n = go(&mut v).map(|n| n > 0);
    println!("{:?} {:?}", String::from_utf8_lossy(&v), n);

    let mut c = Counted(Vec::new(), 0);
    write!(&mut c, "{} {:?} {:>4}", 1, "s", 2.5).unwrap();
    let ok = writeln!(&mut c, "end").is_ok();
    println!("{:?} writes={} ok={}", String::from_utf8_lossy(&c.0), c.1, ok);

    let mut s = String::new();
    write!(&mut s, "{}-{}", 'z', 9).unwrap();
    let mut t = (String::new(), 0u8);
    write!(&mut t.0, "{}", t.1).unwrap();
    println!("{s} {}", t.0);
}
