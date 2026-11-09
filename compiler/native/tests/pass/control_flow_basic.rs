// `std::ops::ControlFlow` (std source, library/core/src/ops.rs): methods,
// derives, and `?` in fns and closures.
use std::collections::HashSet;
use std::ops::ControlFlow;

fn first_neg(xs: &[i32]) -> ControlFlow<i32, usize> {
    let mut seen = 0;
    for &x in xs {
        check(x)?;
        seen += 1;
    }
    ControlFlow::Continue(seen)
}

fn check(x: i32) -> ControlFlow<i32> {
    if x < 0 { ControlFlow::Break(x) } else { ControlFlow::Continue(()) }
}

fn walk(depth: u32, stop: u32) -> ControlFlow<String> {
    if depth == stop {
        return ControlFlow::Break(format!("stop at {depth}"));
    }
    if depth < 5 {
        walk(depth + 1, stop)?;
    }
    ControlFlow::Continue(())
}

fn main() {
    println!("{:?}", first_neg(&[1, 2, 3]));
    println!("{:?}", first_neg(&[1, -2, 3]));
    println!("{:?}", walk(0, 3));
    println!("{:?}", walk(0, 9));
    let c: ControlFlow<&str, u8> = ControlFlow::Continue(7);
    let b: ControlFlow<&str, u8> = ControlFlow::Break("b");
    println!("{} {} {} {}", c.is_continue(), c.is_break(), b.is_continue(), b.is_break());
    println!("{:?} {:?} {:?} {:?}", c.break_value(), c.continue_value(), b.break_value(), b.continue_value());
    println!("{:?} {:?}", c.map_continue(|x| x * 2), b.map_break(|s| s.len()));
    println!("{}", c == ControlFlow::Continue(7));
    let d = c;
    println!("{:?} {:?}", c, d.clone());
    let mut set = HashSet::new();
    set.insert(b);
    set.insert(ControlFlow::Break("b"));
    println!("{}", set.len());
    let sum = |xs: &[i32]| -> ControlFlow<i32, i32> {
        let mut t = 0;
        for &x in xs {
            check(x)?;
            t += x;
        }
        ControlFlow::Continue(t)
    };
    println!("{:?} {:?}", sum(&[4, 5]), sum(&[4, -9, 5]));
    println!("{:?}", visit(&[1, 2, 3, 4], &mut |x| if x * x > 5 { ControlFlow::Break(x) } else { ControlFlow::Continue(()) }));
    let mut seen = Vec::new();
    let _ = visit(&[7, 8], &mut |x| {
        seen.push(x);
        ControlFlow::<()>::Continue(())
    });
    println!("{seen:?}");
}

// The ts-rust visitor shape (astdata `try_for_each_child`).
fn visit<B, F>(xs: &[i32], visitor: &mut F) -> ControlFlow<B>
where
    F: FnMut(i32) -> ControlFlow<B> + ?Sized,
{
    for &x in xs.iter().skip(1) {
        visitor(x)?;
    }
    visitor(xs[0])
}
