// Moving one field out of a struct keeps the other fields usable. A moved field can be
// assigned again, which makes the whole value usable again.
#[derive(Debug)]
struct Pair {
    x: String,
    y: String,
}

#[derive(Debug)]
struct Outer {
    inner: Pair,
    tag: String,
}

fn main() {
    let mut p = Pair { x: String::from("a"), y: String::from("b") };
    let a = p.x;
    p.y.push_str("c");
    println!("{} {}", a, p.y);
    p.x = String::from("d");
    println!("{:?}", p);
    let o = Outer { inner: Pair { x: String::from("e"), y: String::from("f") }, tag: String::from("g") };
    let e = o.inner.x;
    println!("{} {} {}", e, o.inner.y, o.tag);
    if e == "e" {
        let f = o.inner.y;
        println!("{} {}", f, o.tag);
    }
}
