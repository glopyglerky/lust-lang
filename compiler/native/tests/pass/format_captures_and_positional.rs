// Implicit captures next to positional, indexed, named, width and
// precision arguments.
fn main() {
    let a = 1;
    let w = 5;
    println!("{a} {} {a}", 2);
    println!("{a} {0} {0:?}", "z");
    println!("{:w$}|{:.*}|{a:>3}", 7, 2, 1.25);
    println!("{:1$}|{x}", 4, 6, x = a);
    let s = format!("{a}{}{w}", 0);
    println!("{s}");
}
