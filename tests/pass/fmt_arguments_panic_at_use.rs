// A Display impl that panics runs when the `format_args!` value is
// formatted, not when it is made (R9).
use std::fmt;

struct P;

impl fmt::Display for P {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        panic!("boom")
    }
}

fn take(a: fmt::Arguments) {
    println!("took");
    println!("{}", a);
}

fn main() {
    take(format_args!("{}", P));
    println!("end");
}
