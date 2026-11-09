// ARCH 2.8: vtables for generic impl instances, with default methods.
trait Show { fn show(&self) -> String; fn twice(&self) -> String { format!("{}{}", self.show(), self.show()) } }
struct W<T> { x: T }
impl<T: Copy + std::fmt::Debug> Show for W<T> { fn show(&self) -> String { format!("<{:?}>", self.x) } }
struct P<T>(T, T);
impl<T: std::fmt::Debug> Show for P<T> { fn show(&self) -> String { format!("{:?}-{:?}", self.0, self.1) } }
fn main() {
    let a = W { x: 5u8 };
    let b = W { x: 'c' };
    let c = P(1i64, 2i64);
    let all: Vec<&dyn Show> = vec![&a, &b, &c];
    for s in &all { println!("{} {}", s.show(), s.twice()); }
}
