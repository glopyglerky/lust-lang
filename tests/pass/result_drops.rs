// Drop order for Result payloads matched by value (drop elaboration
// must run the taken side's Drop once, and not the other side).
struct Guard { name: String }
impl Drop for Guard { fn drop(&mut self) { println!("drop [{}]", self.name); } }
fn main() {
    let x: Result<Guard, String> = Ok(Guard { name: "g".to_string() });
    match x { Ok(g) => println!("a {}", g.name), Err(e) => println!("e {}", e) }
    let y: Result<Guard, String> = Err("bad".to_string());
    match y { Ok(g) => println!("a {}", g.name), Err(e) => println!("e {}", e) }
    println!("end");
}
