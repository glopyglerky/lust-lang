// A fn item behind `dyn Fn` on one warm daemon: its body edited (the
// vtable entry calls through the slot), a generic fn item, a fn pointer,
// and a `Box<dyn FnOnce>` of a fn item.
fn double(x: u32) -> u32 {
    x * 2 + 1
}
fn id<T>(t: T) -> T {
    t
}
fn main() {
    let f: Box<dyn Fn(u32) -> u32> = Box::new(double);
    let g: &dyn Fn(u32) -> u32 = &double;
    println!("{} {}", f(3), g(4));
}
