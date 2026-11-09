// A closure that returns a borrow of its capture (the closure contract).
fn main() {
    let s = String::from("borrowed");
    let get = || &s;
    println!("{}", get());
}
