// Partial moves of String and Vec fields: the moved buffer is freed
// once, by its new owner (the sanitized difftest checks leaks and
// double frees).
struct P {
    a: String,
    b: String,
    v: Vec<String>,
}

fn main() {
    let p = P { a: String::from("x"), b: String::from("y"), v: vec![String::from("z")] };
    let s = p.b;
    println!("{}", s);
    let mut q = P { a: String::from("1"), b: String::from("2"), v: Vec::new() };
    let t = q.v;
    q.v = vec![String::from("w")];
    println!("{} {} {}", t.len(), q.v.len(), q.a);
    let o = Some((String::from("left"), String::from("right")));
    if let Some((l, _)) = o {
        println!("{}", l);
    }
}
