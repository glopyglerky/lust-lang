// `split_once` (W-STD-B split family).
fn main() {
    let s = "a b";
    println!("{}", s.split_once(' ').unwrap().0);
}
