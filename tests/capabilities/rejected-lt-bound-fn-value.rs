// capability: rejected: lifetimes in `dyn` trait arguments, `T: 'a` bounds on a type's parameters, and fns with `T: 'a` bounds used as values
// expect: L0203

fn f<'a, T: 'a>(x: &'a T) -> &'a T {
    x
}

fn main() {
    let g = f::<u32>;
    println!("{}", g(&1));
}
