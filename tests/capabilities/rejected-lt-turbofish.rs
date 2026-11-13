// capability: rejected: explicit lifetime turbofish args
// expect: L0731

fn f<'a>(x: &'a u32) -> &'a u32 {
    x
}

fn main() {
    let a = 1;
    println!("{}", f::<'static>(&a));
}
