// let-else failure path: scrutinee and earlier locals drop in Rust order.
struct Guard { name: String }
impl Drop for Guard { fn drop(&mut self) { println!("drop [{}]", self.name); } }
fn g(x: i32) -> Option<Guard> { if x > 0 { Some(Guard { name: "s".to_string() }) } else { None } }
fn main() {
    let keep = Guard { name: "keep".to_string() };
    let Some(v) = g(0) else {
        println!("in else");
        return;
    };
    println!("got {}", v.name);
    println!("keep {}", keep.name);
}
