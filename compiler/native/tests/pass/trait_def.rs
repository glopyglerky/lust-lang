trait Greeter {
    fn greet(&self) -> i64;
}

struct S;

impl Greeter for S {
    fn greet(&self) -> i64 {
        1
    }
}

fn main() {
    println!("{}", S.greet());
}
