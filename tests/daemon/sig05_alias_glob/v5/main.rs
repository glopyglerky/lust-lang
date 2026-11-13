mod a;
mod b;
use a::f;

fn main() {
    println!("{} {} {}", f(1), b::h(), b::k());
}
