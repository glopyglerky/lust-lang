// A fn declared in a body is a fn item value like a top-level one.
fn main() {
    fn k(p: u8) -> u8 {
        p + 1
    }
    fn twice(f: fn(u8) -> u8, x: u8) -> u8 {
        f(f(x))
    }
    let as_ptr = k as fn(u8) -> u8;
    let v: Vec<u8> = vec![1u8, 2].into_iter().map(k).collect();
    let g = k;
    println!("{:?} {} {} {}", v, twice(k, 1), g(5), as_ptr(7));
}
