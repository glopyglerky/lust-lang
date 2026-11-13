// A struct of two capture-free closures is Copy when it derives it.
#[derive(Clone, Copy)]
struct Pair<A, B> { a: A, b: B }
fn main() {
    let p = Pair { a: |x: i32| x + 1, b: |x: i32| x * 2 };
    let q = p;
    println!("{} {}", (p.a)(1), (q.b)(3));
}
