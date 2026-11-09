// Option::unwrap_or_default: the payload, or the type's Default value
// (derived, written, or core's) built only on None; through a bound too.
#[derive(Debug)]
struct Cfg(u8);
impl Default for Cfg { fn default() -> Cfg { println!("default!"); Cfg(9) } }
fn g<T: Default>(o: Option<T>) -> T { o.unwrap_or_default() }
fn main() {
    let a: Option<Cfg> = Some(Cfg(1));
    let b: Option<Cfg> = None;
    println!("{:?}", a.unwrap_or_default());
    println!("{:?}", b.unwrap_or_default());
    println!("{} {:?}", g(Some(3u8)), g::<String>(None));
    let v: Option<Vec<(u8, String)>> = None;
    println!("{:?}", v.unwrap_or_default());
}
