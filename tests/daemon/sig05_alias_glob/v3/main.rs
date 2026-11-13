mod a;
mod b;
use a::f as g;

fn main() {
    println!("{} {} {}", g(1), a::h(), b::k());
}
