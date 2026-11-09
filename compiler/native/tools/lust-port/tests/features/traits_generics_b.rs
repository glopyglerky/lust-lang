// feature: traits.generics
// form: associated type with a where clause and a std operator trait impl
trait Source { type Item; fn get(&self) -> Self::Item; }
struct One;
impl Source for One { type Item = i32; fn get(&self) -> i32 { 7 } }
fn fetch<S>(s: &S) -> S::Item where S: Source { s.get() }
#[derive(Debug, PartialEq)]
struct V { x: i32 }
impl std::ops::Add for V { type Output = V; fn add(self, rhs: V) -> V { V { x: self.x + rhs.x } } }
fn main() { println!("{} {:?}", fetch(&One), V { x: 1 } + V { x: 2 }); }
