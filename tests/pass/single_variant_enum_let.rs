// A pattern of a one-variant enum is irrefutable, as rustc's
// exhaustiveness says: `let`, fn and closure params, `for`, and a
// one-arm match.
#[derive(Debug)]
enum One {
    V(u8, String),
}
enum Wrap<T> {
    W { inner: T },
}
fn take(One::V(n, s): One) -> usize {
    n as usize + s.len()
}
fn main() {
    let One::V(a, b) = One::V(3, String::from("xy"));
    println!("{} {}", a, b);
    let Wrap::W { inner } = Wrap::W { inner: 5i64 };
    println!("{}", inner);
    println!("{}", take(One::V(1, String::from("abc"))));
    let f = |One::V(n, _): One| n * 2;
    println!("{}", f(One::V(4, String::new())));
    for One::V(n, s) in vec![One::V(1, String::from("a")), One::V(2, String::from("b"))] {
        println!("{} {}", n, s);
    }
    let r = &One::V(9, String::from("z"));
    let One::V(m, t) = r;
    println!("{} {}", m, t);
    match One::V(7, String::from("q")) {
        One::V(n, s) => println!("{} {}", n, s),
    }
}
