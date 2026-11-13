// Drop order around ? early returns.
struct Guard { name: String }
impl Drop for Guard { fn drop(&mut self) { println!("drop [{}]", self.name); } }
fn f(x: i32) -> Result<i32, String> {
    let a = Guard { name: "a".to_string() };
    let r: Result<Guard, String> = if x > 0 { Ok(Guard { name: "v".to_string() }) } else { Err("no".to_string()) };
    let v = r?;
    println!("have {} {}", a.name, v.name);
    Ok(1)
}
fn main() {
    println!("{:?}", f(1));
    println!("{:?}", f(0));
    println!("done");
}
