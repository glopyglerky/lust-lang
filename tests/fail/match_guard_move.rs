// expect: L0305
struct D(u8);

fn take(d: D) -> u8 {
    d.0
}

fn main() {
    let o = Some(D(5));
    match o {
        Some(x) if take(x) > 10 => println!("big"),
        Some(x) => println!("small {}", x.0),
        None => {}
    }
}
