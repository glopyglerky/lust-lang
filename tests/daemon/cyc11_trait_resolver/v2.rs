const ROUTE: u8 = 0;

struct V(i64);

impl std::ops::Add for V {
    type Output = V;
    fn add(self, o: i64) -> V {
        V(self.0 + o)
    }
}

struct C {
    n: u32,
}

impl Iterator for C {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.n = self.n.checked_sub(1)?;
        Some(self.n)
    }
}

fn show(xs: &[i64]) {
    println!("{:?}", xs);
}

fn main() {
    let _r = ROUTE;
    let v = V(1) + V(2);
    let mut t = 0;
    for x in (C { n: 3 }) {
        t += x;
    }
    show(&[v.0]);
    println!("{}", t);
}
