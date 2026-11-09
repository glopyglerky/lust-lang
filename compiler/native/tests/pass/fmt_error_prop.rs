// fmt::Error from a user impl: write! stops and returns Err, the
// caller's `?` passes it on, partial output stays in the buffer, and
// fmt::Error itself prints through Display and Debug.
use std::fmt;
use std::fmt::Write;

struct Fails;

impl fmt::Display for Fails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("part")?;
        Err(fmt::Error)
    }
}

struct Outer;

impl fmt::Display for Outer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", Fails)?;
        f.write_str("never")
    }
}

fn main() {
    let mut s = String::new();
    let r = write!(s, "a{}b{}", 1, Fails);
    println!("{:?} {:?}", r, s);
    let mut t = String::new();
    let r2 = write!(t, "{}", Outer);
    println!("{:?} {:?} {}", r2, t, t.len());
    println!("[{}] [{:?}] [{:>50}]", fmt::Error, fmt::Error, fmt::Error);
    let ok: fmt::Result = Ok(());
    println!("{:?} {}", ok, matches!(ok, Ok(())));
    let mut u = String::new();
    let r1 = writeln!(u, "x={}", 5);
    let r2 = write!(u, "{:>4}|", "y");
    let r3 = u.write_str("z");
    let r4 = u.write_char('!');
    println!("{:?} {} {:?}", u, u.capacity(), (r1, r2, r3, r4));
}
