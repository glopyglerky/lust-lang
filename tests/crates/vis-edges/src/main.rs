mod a;
mod b;

fn main() {
    println!("vis {}", a::via_inner() + b::read());
}
