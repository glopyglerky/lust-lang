// Moves into functions and structs, reassignment, conditional moves.
#[derive(Debug)]
struct Holder {
    items: Vec<String>,
}

fn consume(s: String) -> usize {
    s.len()
}

fn build(n: usize) -> Holder {
    let mut items = Vec::new();
    let mut i = 0;
    while i < n {
        items.push(format!("item{}", i));
        i += 1;
    }
    Holder { items }
}

fn main() {
    let s = String::from("moved");
    let n = consume(s);
    let mut h = build(3);
    h.items.push(String::from("extra"));
    let mut name = String::from("first");
    if n > 3 {
        name = String::from("second");
    }
    let keep = if h.items.len() > 2 { h } else { build(0) };
    println!("{} {} {:?}", n, name, keep);
    let mut acc = String::new();
    for i in 0..1000 {
        acc = format!("{}", i);
    }
    println!("{}", acc);
}
