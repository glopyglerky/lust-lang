// expect: L0301
struct Pair {
    x: String,
    y: String,
}

fn main() {
    let p = Pair { x: String::from("a"), y: String::from("b") };
    if p.y == "b" {
        let a = p.x;
        println!("{}", a);
    }
    println!("{}", p.x);
}
