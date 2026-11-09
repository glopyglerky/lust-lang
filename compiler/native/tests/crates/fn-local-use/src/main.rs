mod a;
mod b;

fn main() {
    use a::one;
    use b::*;
    println!("local {}", one() + two());
}
