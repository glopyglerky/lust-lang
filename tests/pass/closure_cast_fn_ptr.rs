// A closure cast with `as fn(..)` takes the pointer's signature: its
// params need no annotation, and it may return its argument's borrow
// where the pointer's output is the input's.
static G: u8 = 9;
fn take<T>(f: fn(&u8) -> T, x: &u8) -> T {
    f(x)
}
fn main() {
    let add = (|a| a + 1) as fn(u8) -> u8;
    let id = (|a| a) as fn(&u8) -> &u8;
    let st = (|_a| &G) as fn(&u8) -> &u8;
    let x = 4u8;
    println!("{} {} {} {}", add(1), id(&x), st(&x), take((|a: &u8| *a * 2) as fn(&u8) -> u8, &x));
}
