mod a;
mod b;

use a::*;
use b::*;
use a::value;

fn main() {
    println!("glob {}", value() + only_b());
}
