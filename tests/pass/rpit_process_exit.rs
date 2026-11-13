// A call of an impl Trait fn's bound method in a program that calls
// std::process::exit (hunt2 rpit_call_with_process_exit_ice).
trait Sh { fn area(&self) -> u32; }
struct Sq(u32);
impl Sh for Sq { fn area(&self) -> u32 { self.0 } }
fn mk() -> impl Sh { Sq(9) }
fn main() {
    println!("{}", mk().area());
    std::process::exit(3);
}
