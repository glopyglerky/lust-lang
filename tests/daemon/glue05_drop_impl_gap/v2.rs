struct Ct {
    n: i64,
}
impl Iterator for Ct {
    type Item = i64;
    fn next(&mut self) -> Option<i64> {
        None
    }
}
struct Dg {
    id: i64,
}
pub fn run() {
    println!("end");
}
fn main() {
    run();
}
