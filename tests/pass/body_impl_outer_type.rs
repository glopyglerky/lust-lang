// Impls of a module type written in a fn body apply everywhere, before the fn runs.
struct P { x: i32 }
trait Show { fn show(&self) -> String; }
fn g<T: Show>(t: &T) -> String { t.show() }
fn main() {
    println!("{}", g(&P { x: 3 }));
    println!("{}", P { x: 4 }.double());
}
fn setup() {
    impl Show for P { fn show(&self) -> String { format!("P{}", self.x) } }
    impl P { fn double(&self) -> i32 { self.x * 2 } }
}
