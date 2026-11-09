fn main() {
    let s = base::S::new();
    println!("{} {}", s.get(3), base::read_crate(&s, 4));
}
