// rustc accepts: each field keeps its own lifetime, and the short one may
// end while the long one is still read.
struct Pair<'a, 'b> {
    a: &'a str,
    b: &'b str,
}

impl<'a, 'b> Pair<'a, 'b> {
    fn first(&self) -> &'a str {
        self.a
    }
    fn second(&self) -> &'b str {
        self.b
    }
}

fn keep<'a>(x: &'a str) -> &'a str {
    let r;
    {
        let y = String::from("short");
        let p = Pair { a: x, b: &y };
        println!("{}", p.second());
        r = p.first();
    }
    r
}

fn main() {
    let x = String::from("long");
    println!("{}", keep(&x));
}
