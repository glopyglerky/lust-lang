// print! and println! write each piece to the locked stdout as they
// format it, as std does: a print inside a fmt impl comes out between
// the pieces around it, also through derived Debug, {:#?} padding,
// and eprintln! (W-STD-B).
use std::fmt;

struct Loud(u8);

impl fmt::Debug for Loud {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        println!("inner {}", self.0);
        write!(f, "Loud({})", self.0)
    }
}

impl fmt::Display for Loud {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print!("[d{}]", self.0);
        write!(f, "L{}", self.0)
    }
}

#[derive(Debug)]
struct Pair {
    a: Loud,
    b: Loud,
}

fn main() {
    println!("x {:?} y", Loud(1));
    print!("start ");
    println!("{} and {}", Loud(2), Loud(3));
    println!("{:?}", Pair { a: Loud(4), b: Loud(5) });
    println!("{:#?}", vec![Loud(6)]);
    let s = format!("{:?}", Loud(7));
    println!("after format: {s}");
    eprintln!("e {:?} f", Loud(8));
    println!("{:>6}|{:<4?}|", Loud(9), Some(Loud(10)));
    println!("end");
}
