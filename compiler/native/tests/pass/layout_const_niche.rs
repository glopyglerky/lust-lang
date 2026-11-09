#[derive(Clone, Copy, PartialEq, Debug)]
struct P { p: u64, f: bool }
#[derive(Clone, Copy, PartialEq, Debug)]
enum E { D(P), A, B }
const C1: E = E::A;
const C2: E = E::D(P { p: 40, f: true });
static S: [E; 3] = [E::B, C2, E::D(P { p: 2, f: false })];
const O: Option<bool> = Some(true);
static OS: [Option<bool>; 2] = [None, O];
fn show(e: E) -> u64 { match e { E::D(p) => p.p + p.f as u64, E::A => 100, E::B => 200 } }
fn main() {
    println!("{} {}", show(C1), show(C2));
    for e in S { println!("{}", show(e)); }
    println!("{:?} {:?} {}", C2, S[0], C2 == S[1]);
    println!("{} {}", OS[1].unwrap(), OS[0].is_none());
}
