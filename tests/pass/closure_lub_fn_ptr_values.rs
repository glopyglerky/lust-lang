// Closure values (locals, blocks, nested ifs, derefs) that capture nothing
// join as a fn pointer in if/else, match, arrays, and vec!, and coerce to a
// fn pointer at plain coercion sites (h3-closures: these segfaulted or made
// invalid C when only closure literals became trampolines).
fn one(x: i32) -> i32 {
    x + 100
}

struct S {
    f: fn(i32) -> i32,
}

fn take(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn pick(n: usize) -> fn(i32) -> i32 {
    let c = |x: i32| x * 2;
    let d = |x| x * 3;
    if n == 0 {
        return c;
    }
    if n == 1 { c } else { d }
}

fn main() {
    let n: usize = std::env::args().count();
    let c = |x: i32| x + 1;
    let d = |x: i32| x + 2;
    let e = |x| x + 3;
    let v = if n == 1 { c } else { d };
    let w = match n {
        0 => c,
        1 => { d }
        _ => e,
    };
    let u = if n == 0 { c } else if n == 1 { if n > 5 { d } else { e } } else { d };
    println!("{} {} {}", v(1), w(1), u(1));
    // An array of closure values, a repeated closure, and a fn item.
    let arr = [c, d, e];
    let rep = [c, c, |x| x + 4];
    let mixed = [c, one];
    let mixed2 = [one, c];
    println!("{} {} {} {}", arr[2](10), rep[2](10), mixed[1](1), mixed2[1](1));
    let vv = vec![c, d];
    let annotated: Vec<fn(i32) -> i32> = vec![c, { d }];
    let fixed: [fn(i32) -> i32; 2] = [c, { |x| x * 10 }];
    println!("{} {} {}", vv[1](0), annotated[1](0), fixed[1](3));
    // A fn pointer next to a closure, in both orders.
    let p: fn(i32) -> i32 = |x| x - 1;
    let a = if n == 1 { c } else { p };
    let b = if n == 1 { p } else { c };
    println!("{} {}", a(5), b(5));
    // Plain coercion sites: let, field, tuple, call arg, return.
    let q: fn(i32) -> i32 = c;
    let r: fn(i32) -> i32 = { d };
    let s = S { f: e };
    let t: (fn(i32) -> i32,) = (c,);
    let rc = &c;
    let z = if n == 1 { *rc } else { d };
    println!("{} {} {} {} {} {}", q(1), r(1), (s.f)(1), (t.0)(1), take(e, 1), z(1));
    println!("{} {} {}", pick(0)(5), pick(1)(5), pick(2)(5));
    // A loop or labeled block expected to be a fn pointer.
    let l: fn(i32) -> i32 = loop {
        if n == 1 {
            break c;
        }
        break d;
    };
    let m: fn(i32) -> i32 = 'b: {
        if n == 1 {
            break 'b d;
        }
        e
    };
    println!("{} {}", l(0), m(0));
    // A closure with a reference parameter.
    let f = |s: &str| s.len();
    let g = |s: &str| s.len() * 2;
    let h = if n == 1 { g } else { f };
    println!("{}", h("abc"));
}
