// str::split_at inside a char panics in core (str/mod.rs), with the char-boundary text.
fn main() {
    let s = String::from("héllo");
    let mid = std::hint::black_box(2usize);
    let (a, b) = s.split_at(mid);
    println!("{a}{b}");
}
