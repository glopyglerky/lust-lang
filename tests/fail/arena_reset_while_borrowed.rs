// expect: L0303
// rustc E0502 (ARCH 5.6 "arena reset while borrowed"): `s` borrows the
// arena's storage, so the arena cannot be reset while `s` is live.
struct Arena {
    items: Vec<String>,
}

impl Arena {
    fn get(&self, i: usize) -> &str {
        &self.items[i]
    }
    fn reset(&mut self) {
        self.items.clear();
    }
}

fn main() {
    let mut a = Arena { items: vec![String::from("x")] };
    let s = a.get(0);
    a.reset();
    println!("{}", s);
}
