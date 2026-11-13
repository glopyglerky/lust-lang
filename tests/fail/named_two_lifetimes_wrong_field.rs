// expect: L0733
// rustc: lifetime may not live long enough (`'b` does not outlive `'a`).
struct Pair<'a, 'b> {
    a: &'a str,
    b: &'b str,
}

impl<'a, 'b> Pair<'a, 'b> {
    fn second(&self) -> &'a str {
        self.b
    }
}

fn main() {
    let x = String::from("x");
    let y = String::from("y");
    let p = Pair { a: &x, b: &y };
    println!("{} {}", p.second(), p.a);
}
