// `impl Tr<'static>` methods may take `&'static` where the trait takes `&'d`: the header fixes `'d`.
trait Visit<'d> { fn list(&mut self, l: &'d [u8]) -> usize; }
struct K(usize);
impl Visit<'static> for K { fn list(&mut self, l: &'static [u8]) -> usize { self.0 += l.len(); self.0 } }
static D: [u8; 3] = [1, 2, 3];
fn main() { let mut k = K(0); println!("{}", k.list(&D)); }
