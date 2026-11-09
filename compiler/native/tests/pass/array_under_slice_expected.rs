// An array literal under an expected slice (`&[x]` where `&[Box<dyn Tr>]`
// is expected): each element coerces to the slice's element type, as
// rustc's `check_expr_array` does (fswatch watcher.rs `&[Box::new(..)]`).
trait Opt {
    fn n(&self) -> u8;
}
struct A;
struct B(u8);
impl Opt for A {
    fn n(&self) -> u8 {
        1
    }
}
impl Opt for B {
    fn n(&self) -> u8 {
        self.0
    }
}
fn watch(opts: &[Box<dyn Opt>]) -> u8 {
    opts.iter().map(|o| o.n()).sum()
}
fn names(suffix: &[String]) -> usize {
    suffix.len()
}
fn one() -> u8 {
    1
}
fn two() -> u8 {
    2
}
fn main() {
    println!("{}", watch(&[Box::new(A), Box::new(B(5))]));
    println!("{}", names(&[]));
    let fs: &[fn() -> u8] = &[one, two, || 3];
    println!("{}", fs.iter().map(|f| f()).sum::<u8>());
    let (a, b) = (A, B(2));
    let rs: &mut [&dyn Opt] = &mut [&a, &b];
    rs.swap(0, 1);
    println!("{}", rs[0].n());
    let ws: &[Option<Box<dyn Opt>>] = &[None, Some(Box::new(B(4)))];
    println!("{}", ws.iter().flatten().map(|o| o.n()).sum::<u8>());
}
