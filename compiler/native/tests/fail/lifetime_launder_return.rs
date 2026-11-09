// expect: L0733
struct S {
    v: i32,
}

impl S {
    fn get(&self, other: &i32) -> &i32 {
        let mut r = other;
        r = &self.v;
        r
    }
}

fn main() {
    let s = S { v: 1 };
    let o = 2;
    println!("{}", s.get(&o));
}
