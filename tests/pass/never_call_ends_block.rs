// A call of type `!` does not return, so a value it takes is not used
// after the move on the path that goes on (tsrust T10: `resume_unwind`).
use std::any::Any;
fn die(_b: Box<u8>) -> ! {
    std::process::exit(3)
}
struct K;
impl K {
    fn die(&self, _b: Box<u8>) -> ! {
        std::process::exit(4)
    }
}
fn a(p: Box<u8>) -> Box<u8> {
    if *p == 0 {
        die(p);
    }
    p
}
fn b(p: Box<u8>) -> Box<u8> {
    let _q: u8 = if *p == 0 { K.die(p) } else { 1 };
    p
}
fn c(p: Box<dyn Any + Send>) -> Box<dyn Any + Send> {
    if p.is::<u16>() {
        std::panic::resume_unwind(p);
    }
    p
}
fn d(p: Box<u8>) -> u8 {
    let x: u8;
    if *p > 0 {
        x = 1;
    } else {
        die(p)
    }
    x + *p
}
fn main() {
    println!("{} {} {} {}", a(Box::new(1)), b(Box::new(2)), c(Box::new(3u8)).is::<u8>(), d(Box::new(5)));
}
