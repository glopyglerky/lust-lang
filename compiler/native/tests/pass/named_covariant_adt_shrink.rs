// rustc accepts: `Co<'a>` is covariant in 'a, so `Co<'b>` shrinks to `Co<'a>`.
struct Co<'a> {
    r: &'a i32,
}

fn shrink<'a, 'b: 'a>(x: Co<'b>) -> Co<'a> {
    x
}

fn main() {
    let one = 1;
    let c = shrink(Co { r: &one });
    println!("{}", c.r);
}
