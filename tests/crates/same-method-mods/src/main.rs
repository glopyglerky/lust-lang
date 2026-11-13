mod a;
mod b;

fn main() {
    println!("same-method {} {}", crate::a::run(), crate::b::run());
}
