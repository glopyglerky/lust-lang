// `T: 'static` holds for string literals, a `&'static` global, and owned
// values.
static NAME: &str = "global";
fn keep<T: 'static + Copy>(t: T) -> T { t }
fn main() {
    let a = keep("lit");
    let b = keep(NAME);
    let n = keep(5u32);
    println!("{a} {b} {n}");
}
