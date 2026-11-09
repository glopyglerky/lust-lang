use std::fmt::Debug;
struct Stack<T> { items: Vec<T> }
impl<T: Clone + Debug> Stack<T> {
    fn push(&mut self, x: T) -> usize { self.items.push(x); self.items.len() }
    fn iter(&self) -> impl Iterator<Item = T> + '_ { self.items.iter().cloned() }
}
fn f(x: u32, y: u8) -> u32 { x + 1 }
fn user(v: u32) -> u32 { f(v) * 2 }
fn other() -> String { format!("{:?}", f(2)) }
fn fill(s: &mut Stack<String>) -> usize { s.push("a".to_string()) + s.push(String::new()) }
fn walk(s: &Stack<String>) -> usize { s.iter().count() }
fn main() {
    let mut s = Stack { items: Vec::new() };
    let n = fill(&mut s);
    println!("{} {} {} {} {}", user(3), other(), f(1), n, walk(&s));
}
