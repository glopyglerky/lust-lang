mod other { #[derive(Debug)] pub enum State { Q, R } pub fn f() -> State { State::R } }
fn run() -> usize {
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum State { NotTaken, Done }
    use State::*;
    let v = vec![NotTaken; 3];
    v.iter().filter(|s| **s == Done).count() + v.len()
}
fn main() { println!("{} {:?}", run(), other::f()); }
