// ARCH 2.8: `&dyn` is two words; `Option<&dyn>` uses the pointer niche.
trait Tr { fn v(&self) -> u8; }
impl Tr for u8 { fn v(&self) -> u8 { *self } }
fn main() {
    println!("{} {} {}", std::mem::size_of::<&dyn Tr>(), std::mem::size_of::<Option<&dyn Tr>>(), std::mem::align_of::<&mut dyn Tr>());
    let x = 3u8;
    let o: Option<&dyn Tr> = Some(&x as &dyn Tr);
    match o { Some(d) => println!("{}", d.v()), None => println!("none") }
    let n: Option<&dyn Tr> = None;
    println!("{}", n.is_none());
}
