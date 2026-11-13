#[allow(dead_code)]
#[derive(Debug)]
enum State { Q, R }
fn run() -> usize {
    #[allow(dead_code)]
    enum State { NotTaken, Waiting, Done }
    use State::Waiting;
    let v = [Waiting, State::Done];
    v.len()
}
fn main() { println!("{} {:?}", run(), State::Q); }
