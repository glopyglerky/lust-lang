// `fmt::Arguments` implements Display and Debug (R9): both write the
// formatted text as it is, width and precision ignored. A
// `format_args!` value in every position: stored, passed to a user
// generic, a struct field, `Option<Arguments>` (the gostd
// `assert_failed` shape), `to_string` (capacity included), and
// argument temporaries extended to the end of the block.
use std::fmt::{self, Debug, Display};

fn show(x: impl Display) -> String {
    format!("{}", x)
}

fn dbg(x: impl Debug) -> String {
    format!("{:?}", x)
}

fn assert_failed(message: Option<fmt::Arguments<'_>>) -> String {
    match message {
        Some(m) => format!("False expression: {m}"),
        None => "False expression.".to_string(),
    }
}

struct Wrap<'a>(fmt::Arguments<'a>);

impl Display for Wrap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.0)
    }
}

struct T(&'static str);

impl Drop for T {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

impl Display for T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T{}", self.0)
    }
}

fn main() {
    let x = std::env::args().count();
    println!("{}", show(format_args!("hi {} {:03}", "x", 7)));
    println!("{}", dbg(format_args!("d {}", 2)));
    println!("{}", assert_failed(Some(format_args!("{} things", x))));
    println!("{}", assert_failed(None));

    let a = format_args!("ab{}", x);
    println!("[{:>10}][{:<10?}][{:.2}][{:#?}]", a, a, a, a);
    println!("{:?}", format_args!("{}{:?}{:x}", 1, "s", 255));
    println!("[{}]", format_args!(""));

    let s = format_args!("ab{} cd{}", x, "hello").to_string();
    println!("{:?} len={} cap={}", s, s.len(), s.capacity());
    let s = format_args!("plain literal").to_string();
    println!("len={} cap={}", s.len(), s.capacity());

    println!("{}", Wrap(format_args!("w{}", 1)));
    let v = vec![format_args!("one"), format_args!("two {}", 2)];
    for e in &v {
        println!("{}", e);
    }
    let c = a.clone();
    println!("{} {} {}", a, c, a);
    println!("{}", format!("{:?}", Some(format_args!("in opt {}", 9))));
    println!("{}", format_args!("outer [{}]", format_args!("inner {}", 3)));

    // Argument temporaries live to the end of the block.
    let t = format_args!("{}", T("x"));
    println!("{}", t);
    {
        let u = format_args!("{}", T("y"));
        println!("{}", u);
        println!("inner end");
    }
    println!("outer end");
    println!("{}", t);
}
