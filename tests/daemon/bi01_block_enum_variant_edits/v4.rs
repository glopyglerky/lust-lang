#[allow(dead_code)]
#[derive(Debug)]
enum State { Q, R }
fn run() -> usize {
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum State { NotTaken, Waiting, Done }
    use State::Waiting;
    let mut v = vec![State::NotTaken; 3];
    v[0] = Waiting;
    v.iter().filter(|s| matches!(s, Waiting | State::Done)).count() * 10 + v.len()
}
fn main() { println!("{} {:?}", run(), State::Q); }
