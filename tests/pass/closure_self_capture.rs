// Methods: closures capture fields reached through `self`.
struct Counter {
    step: u32,
    hits: u32,
    name: String,
}
impl Counter {
    fn scaled(&self, xs: [u32; 3]) -> u32 {
        let f = |x: u32| x * self.step;
        f(xs[0]) + f(xs[1]) + f(xs[2])
    }
    fn bump(&mut self) {
        let mut hit = || self.hits += self.step;
        hit();
        hit();
    }
    fn label(&self) -> String {
        let l = || format!("{}:{}", self.name, self.hits);
        l()
    }
}
fn main() {
    let mut c = Counter { step: 2, hits: 0, name: String::from("c") };
    println!("{}", c.scaled([1, 2, 3]));
    c.bump();
    println!("{} {}", c.hits, c.label());
}
