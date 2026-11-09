// rustc flattens a `format_args!` argument that one Display or Debug
// placeholder uses into the outer format (R9): the outer literals,
// capacity estimate, literal inlining, and `'static` text follow the
// flattened form, and the inner arguments evaluate in place.
use std::fmt;

fn stat() -> fmt::Arguments<'static> {
    format_args!("{} {:?}", format_args!("x{}", 1), format_args!("y"))
}

fn arg(n: i32) -> i32 {
    println!("eval {n}");
    n
}

fn main() {
    let x = 5;
    println!("{}", stat());
    let a = format_args!("outer [{}] {}", format_args!("inner {} {:?}", x, "q"), x);
    let s = a.to_string();
    println!("{} {} {}", s, s.len(), s.capacity());
    let f = format!("{:>9}|{}", format_args!("{}{}", arg(1), arg(2)), arg(3));
    println!("{f:?} {}", f.capacity());
    let g = format!("{}", format_args!("{}", String::from("tmp")));
    println!("{g} {}", g.capacity());
    let h = format_args!("{0} {0}", format_args!("twice {}", x)).to_string();
    println!("{h} {}", h.capacity());
    let t = format_args!("{}", &&format_args!("refs {}", x)).to_string();
    println!("{t} {}", t.capacity());
}
