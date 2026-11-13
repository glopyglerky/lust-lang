// `dyn Error` (F3): the object type implements Debug and Display
// through the vtable fmt shims, calls its own methods, and walks
// a `source` chain to the end. Was tests/reject (L0702) before F3.
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct Leaf(u32);
impl fmt::Display for Leaf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "leaf {}", self.0) }
}
impl Error for Leaf {}

#[derive(Debug)]
struct Wrap(Box<dyn Error>);
impl fmt::Display for Wrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "wrap({})", self.0) }
}
impl Error for Wrap {
    fn source(&self) -> Option<&(dyn Error + 'static)> { Some(&*self.0) }
}

fn show(e: &dyn Error) {
    print!("{e}");
    let mut cur = e.source();
    while let Some(s) = cur {
        print!(" <- {s} [{s:?}]");
        cur = s.source();
    }
    println!();
}

fn show_ss(e: &(dyn Error + Send + Sync)) {
    println!("ss: {e}");
}

fn main() {
    show(&Leaf(1));
    show(&Wrap(Box::new(Leaf(2))));
    show(&Wrap(Box::new(Wrap(Box::new(Leaf(3))))));
    let b: Box<dyn Error> = Box::new(Leaf(4));
    show(&*b);
    show_ss(&Leaf(5));
    #[allow(deprecated)]
    match Wrap(Box::new(Leaf(6))).cause() {
        Some(c) => println!("cause: {c}"),
        None => println!("cause: none"),
    }
    let owned: Box<dyn Error + Send + Sync> = Box::new(Leaf(7));
    show_ss(&*owned);
}
