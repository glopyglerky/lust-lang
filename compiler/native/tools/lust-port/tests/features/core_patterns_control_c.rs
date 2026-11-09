// feature: core.patterns_control
// form: let without init, ref bindings, or, range and nested enum patterns
enum E { A(i32), B }
fn probe(e: E) -> i32 {
    match e {
        E::A(ref n) => *n,
        E::B => 0,
    }
}
fn main() {
    let x: i32;
    x = 4;
    let pair = (1, 2);
    let (ref a, b) = pair;
    let score = match x {
        0 | 1 => 100,
        2..=9 => 10,
        _ => 0,
    };
    let outer = Some(E::A(3));
    let inner = match &outer {
        Some(E::A(n)) => *n,
        _ => 0,
    };
    println!("{} {} {} {}", x + *a + b, score, inner, probe(E::A(7)));
}
