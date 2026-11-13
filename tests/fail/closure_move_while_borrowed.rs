// expect: L0311
fn main() {
    let s = String::from("a");
    let c = || s.len();
    let t = s;
    println!("{} {}", c(), t);
}
