// Format arguments are evaluated once each, in argument order, before formatting.
fn f() -> u32 { println!("called"); 3 }
fn main1() { println!("{0} {0} {1}", f(), f()); let s = format!("{0}-{0}", f()); println!("{}", s); }
fn g(x: u32) -> u32 { print!("[f{}]", x); x }
fn main() { main1(); println!("{1} {0}", g(1), g(2)); let s = format!("{1}{0}{1}", g(3), g(4)); println!("{}", s); }
