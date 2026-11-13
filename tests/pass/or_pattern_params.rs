enum E { A(i32), B(i32) }

fn first(((x, _) | (_, x)): (i32, i32)) -> i32 { x }
fn sum((a, b): (i32, i32)) -> i32 { a + b }
fn unwrap_e((E::A(x) | E::B(x)): E) -> i32 { x }
fn wild(_: i32, y: i32) -> i32 { y }
fn deref(&x: &i32) -> i32 { x }
fn at_param(w @ (a, b): (i32, i32)) -> i32 { w.0 + a + b }
fn ref_param(ref s: String) -> usize { s.len() }

fn main() {
    println!("{}", first((7, 8)));
    println!("{}", sum((1, 2)));
    println!("{} {}", unwrap_e(E::A(3)), unwrap_e(E::B(4)));
    println!("{}", wild(9, 5));
    println!("{}", deref(&6));
    println!("{}", at_param((1, 2)));
    println!("{}", ref_param("hello".to_string()));
    let g = |((x, _) | (_, x)): (i32, i32)| x * 2;
    println!("{}", g((11, 22)));
}
