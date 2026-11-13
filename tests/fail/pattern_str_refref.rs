// expect: L0110
fn main() {
    let s: &str = "ab";
    println!("{}", matches!(&s, "ab"));
}
