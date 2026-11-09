// std::io::Write (W-SHIM): write!/writeln! into stdout, stderr, Vec<u8>,
// sink, `&mut dyn Write`, and generic writers; write_all, flush, by_ref,
// and locks. Stdout writes interleave with println! in program order.
use std::io::{self, Write};

fn emit(w: &mut dyn Write, n: i32) -> io::Result<()> {
    writeln!(w, "dyn {} {:>4} {:?}", n, "x", Some(n))?;
    w.write_all(b"raw\n")?;
    Ok(())
}

fn emit_generic<W: Write>(mut w: W, s: &str) -> io::Result<usize> {
    write!(w, "[{}]", s)?;
    writeln!(w)?;
    w.flush()?;
    Ok(s.len())
}

fn report(out: &mut impl Write, items: &[(&str, u32)]) -> io::Result<()> {
    for i in 0..items.len() {
        let (name, n) = items[i];
        writeln!(out, "{name:<6}|{n:>3}")?;
    }
    Ok(())
}

fn main() {
    let mut out = io::stdout();
    println!("first");
    let _ = writeln!(out, "hello {}", 42);
    print!("partial ");
    let _ = out.write_all(b"bytes\n");
    let _ = out.flush();
    let mut v: Vec<u8> = Vec::new();
    let _ = write!(v, "a{}b", 1);
    let _ = writeln!(&mut v, "c");
    let n = v.write(b"xyz");
    println!("{:?} {:?}", v, n);
    let _ = emit(&mut out, 7);
    let _ = emit(&mut v, 8);
    println!("{}", v.len());
    match emit_generic(&mut v, "g") {
        Ok(k) => println!("generic {k} {}", v.len()),
        Err(e) => println!("error {e}"),
    }
    let _ = emit_generic(io::stdout(), "to stdout");
    let _ = report(&mut out, &[("ab", 1), ("cdef", 234)]);
    let _ = report(&mut v, &[("z", 9)]);
    let mut s = io::sink();
    let _ = writeln!(s, "gone {}", 1);
    let _ = s.write_all(b"also gone");
    let mut err = io::stderr();
    let _ = writeln!(err, "err {}", 1);
    let _ = writeln!(io::stdout().lock(), "locked {}", true);
    let _ = writeln!(io::stderr().lock(), "locked err");
    let mut lock = out.lock();
    let _ = write!(lock.by_ref(), "by_ref ");
    let _ = writeln!(lock, "done");
    let text = ascii(&v);
    println!("{}", text);
}

fn ascii(v: &[u8]) -> String {
    let mut s = String::new();
    for &b in v {
        s.push(b as char);
    }
    s
}
