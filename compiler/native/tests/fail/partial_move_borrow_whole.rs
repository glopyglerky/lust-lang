// expect: L0301
struct Pair {
    x: String,
    y: String,
}

fn read(p: &Pair) -> usize {
    p.y.len()
}

fn main() {
    let p = Pair { x: String::from("a"), y: String::from("b") };
    let a = p.x;
    println!("{} {}", a, read(&p));
}
