// std::error::Error in source (W-STD-C phase 2, F3): plain impls,
// a bound use, the deprecated description() default text, and a
// `source` chain. `dyn Error` itself is std_error_dyn.rs.
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct Leaf(i32);

impl fmt::Display for Leaf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "leaf {}", self.0)
    }
}

impl Error for Leaf {}

#[derive(Debug)]
struct Wrap;

impl fmt::Display for Wrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wrap")
    }
}

impl Error for Wrap {
    fn description(&self) -> &str {
        "a wrapped thing"
    }
}

#[derive(Debug)]
struct Outer(Leaf);

impl fmt::Display for Outer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "outer")
    }
}

impl Error for Outer {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

fn generic<E: Error>(e: &E) -> String {
    format!("{}|{:?}", e, e)
}

fn main() {
    println!("{}", generic(&Leaf(9)));
    println!("{}", generic(&Wrap));
    println!("desc: {} / {}", Leaf(1).description(), Wrap);
    let o = Outer(Leaf(7));
    println!("src: {} / {:?}", o.source().unwrap(), o.source().unwrap().source());
}
