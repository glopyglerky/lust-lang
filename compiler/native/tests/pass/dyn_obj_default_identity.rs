// `&'a dyn T` means `&'a (dyn T + 'a)`: identity fns keep the default
trait T { fn r(&self) -> u32; }
impl T for u32 { fn r(&self) -> u32 { *self } }
fn id<'a>(x: &'a dyn T) -> &'a dyn T { x }
fn id2(x: &dyn T) -> &dyn T { x }
fn main() { let n = 1u32; println!("{} {}", id(&n).r(), id2(&n).r()); }
