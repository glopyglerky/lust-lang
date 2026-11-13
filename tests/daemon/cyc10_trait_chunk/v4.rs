struct V(i64);

impl PartialEq for V {
    fn eq(&self, o: &Self) -> bool {
        self.0 == o.0
    }
}

impl std::fmt::Display for V {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

fn show(xs: &[i64]) {
    println!("{}", xs.len());
}

fn main() {
    let (a, b) = (V(1), V(1));
    show(&[a.0]);
    println!("{} {}", a == b, a);
}
