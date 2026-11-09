// expect: L0305
struct Guard {
    name: String,
    n: i32,
}

impl Drop for Guard {
    fn drop(&mut self) {
        println!("drop {}", self.name);
    }
}

fn main() {
    let g = Guard { name: "x".to_string(), n: 1 };
    let s = g.name;
    println!("{}", s);
}
