mod other { #[derive(Debug)] pub enum State { Q, R } pub fn f() -> State { State::R } }
fn run() -> usize {
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum State { NotTaken, Waiting, Done }
    use State::*;
    let mut v = vec![NotTaken; 3];
    v[0] = Waiting;
    v[1] = Done;
    v.iter().filter(|s| matches!(s, Waiting | Done)).count() * 10 + v.len()
}
fn main() { println!("{} {:?}", run(), other::f()); }
