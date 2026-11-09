// A `[T; 0]` returned through `Default` glue and a fn call, several times.
fn mk() -> [u16; 0] {
    Default::default()
}
fn main() {
    let a = mk();
    let b = mk();
    let c = mk();
    let d = mk();
    println!("{:?}{:?}{:?}{:?}", a, b, c, d);
    println!("{:?}", <[u16; 0]>::default());
}
