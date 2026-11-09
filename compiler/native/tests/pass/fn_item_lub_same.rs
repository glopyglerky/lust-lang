// rustc's LUB keeps one fn item type when both sides name the same
// item (args that unify): zero-sized, not a fn pointer.
fn foo(x: u8) -> u8 {
    x + 1
}
fn id<T>(x: T) -> T {
    x
}
enum E {
    A(u8),
}
fn main() {
    let c = std::env::args().count() > 0;
    let f = if c { foo } else { foo };
    let g = match c {
        true => foo,
        false => foo,
    };
    let a = [id, id::<u8>];
    let b = [id::<u8>, id, foo];
    let d = if c { id } else { id::<u16> };
    let h = [E::A, E::A];
    let k = match c {
        true => E::A,
        false => E::A,
    };
    println!("{} {} {}", f(1), std::mem::size_of_val(&f), std::mem::size_of_val(&g));
    println!("{} {} {} {}", std::mem::size_of_val(&a), std::mem::size_of_val(&b), std::mem::size_of_val(&d), d(3));
    println!("{} {} {}", std::mem::size_of_val(&h), std::mem::size_of_val(&k), b[2](4));
    let _ = (a[0](1), h[0](3), k(4), g(5));
}
