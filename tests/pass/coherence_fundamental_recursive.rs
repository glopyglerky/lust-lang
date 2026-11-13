// Coherence (ARCH 2.6): a blanket `impl<T: Clone> Tr for T` does not
// overlap `impl Tr for Box<My>` when `My` is local and not Clone: core's
// `Clone for Box<T>` needs `My: Clone`, which cannot hold (Box is
// fundamental, so no downstream crate can add it either). Nested boxes too.
trait Tr { fn v(&self) -> u8; }
impl<T: Clone> Tr for T { fn v(&self) -> u8 { 1 } }
struct My;
impl Tr for Box<My> { fn v(&self) -> u8 { 2 } }
struct Other;
impl Tr for Box<Box<Other>> { fn v(&self) -> u8 { 3 } }
fn main() { println!("{} {} {}", Box::new(My).v(), Box::new(Box::new(Other)).v(), 5u8.v()); }
