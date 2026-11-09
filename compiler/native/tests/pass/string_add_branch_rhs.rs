// `String + x`: std's one impl is `Add<&str>`, so the right operand is checked
// as a `&str` (an if or match of literals, `&String` by deref coercion).
fn main() {
    let c = std::env::args().count() > 5;
    let t = String::from("t");
    let s = String::from("a") + if c { "" } else { ">" };
    let u = String::from("b") + match c { true => "x", false => "y" } + &t + if c { &t } else { "z" };
    println!("{s} {u}");
}
