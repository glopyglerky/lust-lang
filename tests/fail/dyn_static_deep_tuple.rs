// expect: L0733
// rustc: lifetime may not live long enough. The `'static` object lifetime
// sits 32 tuples deep; the walk must reach it, not stop and accept.
trait T {}
fn bad(x: ((((((((((((((((((((((((((((((((&dyn T,),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),)) -> ((((((((((((((((((((((((((((((((&(dyn T + 'static),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),),) {
    x
}
fn main() {}
