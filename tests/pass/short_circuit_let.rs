// Short-circuit operators as `let` initializers, nested and inside `if`/`match` initializers.
fn side(tag: &str, v: bool) -> bool {
    println!("side {tag}");
    v
}

fn main() {
    let a = true;
    let b = false;
    let hot = a && b;
    let cold = a || b;
    let deep = (a && b) || (b && a) || a;
    let via_if = if a { a && b } else { b || a };
    let via_match = match a {
        true => b && a,
        false => a || b,
    };
    let skipped = b && side("rhs-and", true);
    let kept = a || side("rhs-or", false);
    let run = a && side("run-and", true);
    println!("{hot} {cold} {deep} {via_if} {via_match} {skipped} {kept} {run}");
    let mut m = a && b;
    m = b || a;
    if a && side("cond", true) {
        println!("in {m}");
    }
}
