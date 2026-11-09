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
impl Drop for Dg {
    fn drop(&mut self) {
        println!("drop {}", self.id);
    }
}
pub fn run() {
    let v = vec![Dg { id: 5 }, Dg { id: 6 }];
    println!("vl {}", v.len());
    println!("end");
}
fn main() {
    run();
}
