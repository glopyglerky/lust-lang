// Closure params and `if let`/`while let` bindings shadow an imported fn
// or a macro of the same name in their bodies (the resolver bound them
// over nothing, so `helper(3)` called the imported fn).
mod m {
    pub fn helper(x: u8) -> u8 {
        x + 100
    }
}
use m::helper;

macro_rules! args {
    () => {
        0
    };
}

fn double(x: u8) -> u8 {
    x * 2
}

fn main() {
    let g = |helper: fn(u8) -> u8| helper(3);
    println!("{}", g(double));
    let o: Option<fn(u8) -> u8> = Some(double);
    if let Some(helper) = o {
        println!("{}", helper(4));
    }
    if let Some(helper) = o
        && helper(1) == 2
    {
        println!("{}", helper(5));
    } else {
        println!("{}", helper(5));
    }
    let mut w: Option<fn(u8) -> u8> = Some(double);
    while let Some(helper) = w {
        println!("{}", helper(6));
        w = None;
    }
    let none: Option<fn(u8) -> u8> = None;
    if let Some(helper) = none {
        println!("{}", helper(7));
    } else {
        println!("{}", helper(7));
    }
    let f = |args: Vec<String>| args.len();
    println!("{} {} {}", f(vec![String::new()]), args!(), helper(1));
}
