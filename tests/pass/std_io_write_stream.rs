// `io::Write::write_fmt` on stdout, stderr, and their locks streams
// each part to the locked stream as it formats, as std does: a print
// inside a fmt impl comes out between the parts around it. Argument
// expressions still run before the first part (they are evaluated
// when the `format_args!` value is made), and `write!` into a String
// or a Vec<u8> keeps building its text first.
use std::fmt;
use std::fmt::Write as _;
use std::io::{self, Write};

struct Loud(u8);

impl fmt::Debug for Loud {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print!("[out{}]", self.0);
        eprint!("[err{}]", self.0);
        write!(f, "Loud({})", self.0)
    }
}

impl fmt::Display for Loud {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        println!("inner {}", self.0);
        write!(f, "L{}", self.0)
    }
}

#[derive(Debug)]
struct Pair {
    a: Loud,
    b: Loud,
}

fn side() -> u8 {
    print!("SIDE ");
    7
}

fn emit<W: Write>(w: &mut W, v: &Loud) {
    let _ = writeln!(w, "gen {:?} end", v);
}

fn main() {
    let _ = write!(io::stdout(), "pre {:?} post\n", Loud(1));
    let _ = writeln!(io::stdout(), "disp {} end", Loud(2));
    let _ = write!(io::stderr(), "epre {:?} epost\n", Loud(3));
    let _ = writeln!(io::stderr(), "edisp {} eend", Loud(4));
    let mut lock = io::stdout().lock();
    let _ = write!(lock, "lock {:?} ", Loud(5));
    let _ = writeln!(&mut lock, "ref {:?} done", Loud(6));
    let _ = write!(lock.by_ref(), "by {:?}\n", Loud(7));
    let mut elock = io::stderr().lock();
    let _ = writeln!(elock, "elock {:?} done", Loud(8));
    let mut out = io::stdout();
    let _ = out.write_fmt(format_args!("direct {:?} {}\n", Loud(9), side()));
    emit(&mut out, &Loud(10));
    emit(&mut io::stderr(), &Loud(11));
    let _ = writeln!(out, "{:?}", Pair { a: Loud(12), b: Loud(13) });
    let _ = writeln!(out, "{:#?}", vec![Loud(14)]);
    let _ = writeln!(out, "{:>9}|{:<6?}|", Loud(15), Some(Loud(16)));
    let _ = writeln!(Box::new(io::stdout()), "boxed {:?}", Loud(17));
    // Not streams: the text is built first, so the inner prints come
    // out before the value is shown here.
    let mut v: Vec<u8> = Vec::new();
    let _ = write!(v, "v{:?}w", Loud(18));
    println!("vec {}", v.len());
    let mut s = String::new();
    let _ = write!(s, "s{}t", Loud(19));
    println!("str {s}");
    eprintln!("edone");
    println!("end");
}
