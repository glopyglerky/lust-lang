mod other { #[derive(Debug)] pub enum State { Q, R } pub fn f() -> State { State::R } }
fn run() -> usize {
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum State { NotTaken, Waiting, Done }
    let mut v = vec![State::NotTaken; 3];
    v[0] = State::Waiting;
    v.iter().filter(|s| matches!(s, State::Waiting | State::Done)).count() * 10 + v.len()
}
fn main() { println!("{} {:?}", run(), other::f()); }
