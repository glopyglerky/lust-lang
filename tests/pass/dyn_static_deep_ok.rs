// A `'static` object lifetime shortens to an erased one 40 tuples deep.
trait T {
    fn r(&self) -> u32;
}
impl T for u32 {
    fn r(&self) -> u32 {
        *self
    }
}
fn main() {
    static N: u32 = 3;
    let d: &'static dyn T = &N;
    let x = ((((((((((((((((((((((((((((((((((((((((d,),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),);
    let y: ((((((((((((((((((((((((((((((((((((((((&dyn T,),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),) = x;
    let _ = y;
    println!("{}", d.r());
}
