mod a;
fn main() {
    let s = a::S::new();
    println!("{} {}", s.get(3), a::read(&s, 4));
}
