macro_rules! println { ($($t:tt)*) => { std::print!("custom\n") }; }
macro_rules! vec { ($($t:tt)*) => { 42 }; }
macro_rules! assert_eq { ($a:expr, $b:expr) => { std::println!("mine {} {}", $a, $b) }; }

fn main() {
    println!("hi");
    let v: i32 = vec![1, 2];
    std::println!("{}", v);
    assert_eq!(1, 2);
}
