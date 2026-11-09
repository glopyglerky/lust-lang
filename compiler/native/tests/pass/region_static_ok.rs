// rustc accepts: 'static data in 'static slots, elision through one input
// and through `&self`.
struct Named {
    name: &'static str,
    n: i32,
}

impl Named {
    fn name(&self) -> &str {
        self.name
    }
    fn label(&self, _other: &str) -> &str {
        self.name
    }
}

fn keep(x: &'static str) -> usize {
    x.len()
}

fn first(v: &Vec<String>) -> &str {
    &v[0]
}

fn pick() -> &'static str {
    "pick"
}

fn main() {
    let a = Named { name: "a", n: 1 };
    let s = String::from("other");
    println!("{} {} {} {}", a.name(), a.label(&s), keep("abc"), keep(pick()));
    let v = vec![String::from("x")];
    let f = first(&v);
    println!("{} {}", f, a.n);
}
