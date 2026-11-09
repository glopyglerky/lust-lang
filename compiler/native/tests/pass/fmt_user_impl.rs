// User `impl Display` and `impl Debug`: dispatch from `{}` and `{:?}`
// with specs (write_str ignores width; pad and leaf fmt calls use it),
// write!/writeln! to a Formatter, the Formatter queries, `?` on
// fmt::Result, nesting in derived Debug, and to_string.
use std::fmt;

struct Cat(u8);

impl fmt::Display for Cat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => f.write_str("none"),
            n => write!(f, "Cat({n})"),
        }
    }
}

impl fmt::Debug for Cat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

struct Padded(&'static str);

impl fmt::Display for Padded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.0)
    }
}

struct Probe;

impl fmt::Display for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "w={:?} p={:?} fill={:?} alt={} plus={} minus={} zero={}", f.width(), f.precision(), f.fill(), f.alternate(), f.sign_plus(), f.sign_minus(), f.sign_aware_zero_pad())?;
        writeln!(f)?;
        writeln!(f, "line {}", 2)
    }
}

#[derive(Debug)]
struct Holder {
    cat: Cat,
    cats: Vec<Cat>,
    maybe: Option<Cat>,
}

struct Wrap<T>(T);

impl<T: fmt::Display> fmt::Display for Wrap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0)
    }
}

fn main() {
    let c = Cat(3);
    println!("{} {} [{:>8}] [{:<8?}] [{:08?}]", c, Cat(0), c, c, Cat(7));
    println!("[{:>6}] [{:<6}] [{:^6.2}] [{}]", Padded("ab"), Padded("cd"), Padded("xyz"), Padded("plain"));
    print!("{}", Probe);
    print!("{:>+#09.3}", Probe);
    print!("{:*<-4}", Probe);
    let h = Holder { cat: Cat(1), cats: vec![Cat(2), Cat(0)], maybe: Some(Cat(9)) };
    println!("{:?}", h);
    println!("{:#?}", h);
    println!("{:5?}", h.cats);
    println!("{} {}", Wrap(5), Wrap(Cat(4)));
    println!("{}", Wrap(Wrap("nested")));
    let s = c.to_string();
    println!("{} {} {}", s, s.len(), s.capacity());
    let s2 = format!("{}{}", c, Cat(0));
    println!("{} {}", s2, s2.capacity());
}
