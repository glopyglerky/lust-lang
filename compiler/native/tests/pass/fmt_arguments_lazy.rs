// A `format_args!` value formats its arguments at each use, as in
// rustc (R9): a Display impl with a side effect runs once per use and
// never when the value is unused, a print inside one comes out where
// the value is formatted, an error from one reaches the outer
// formatter, and io::Write::write_fmt formats when it writes.
use std::cell::Cell;
use std::fmt::{self, Display, Write as _};
use std::io::Write as _;

struct C<'a>(&'a Cell<u32>);

impl Display for C<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.set(self.0.get() + 1);
        write!(f, "c{}", self.0.get())
    }
}

struct P;

impl Display for P {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        println!("P::fmt runs");
        write!(f, "P")
    }
}

struct E;

impl Display for E {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Err(fmt::Error)
    }
}

fn twice<T: Display + Copy>(t: T) -> String {
    format!("{t}|{t}")
}

fn show(a: fmt::Arguments) {
    println!("show starts");
    println!("{}", a);
}

fn main() {
    let n = Cell::new(0);
    let a = format_args!("{}", C(&n));
    println!("{} {}", a, a);
    let _unused = format_args!("{}", C(&n));
    println!("count {}", n.get());
    println!("{}", twice(format_args!("<{}>", C(&n))));
    show(format_args!("{}", P));

    let mut s = String::new();
    let r = write!(s, "x{}y", format_args!("a{}b", E));
    println!("{:?} {:?}", r.is_err(), s);

    let w = 6;
    let p = 2;
    let b = format_args!("{:>w$.p$}|{:<1$}|{:^1$}", 3.14159, 7, "ab");
    println!("[{}] [{:?}] [{:>40}]", b, b, b);
    let t = b.to_string();
    println!("{t:?} {} {}", t.len(), t.capacity());

    let mut v: Vec<u8> = Vec::new();
    write!(v, "a{}b{}", C(&n), C(&n)).unwrap();
    writeln!(v, "!{}", 1).unwrap();
    println!("{:?} {}", String::from_utf8(v).unwrap(), n.get());
}
