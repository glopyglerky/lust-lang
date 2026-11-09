// capability: rejected: lifetimes in `dyn` trait arguments, `T: 'a` bounds on a type's parameters, and fns with `T: 'a` bounds used as values
// expect: L0203

struct S<'a, T: 'a> {
    x: &'a T,
}

fn main() {
    let a = 1;
    let s = S { x: &a };
    println!("{}", s.x);
}
