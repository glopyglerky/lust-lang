// feature: attrs.unsupported
// form: a linker attribute lust does not know (used)
#[used]
static X: i32 = 3;
fn main() { println!("{}", X); }
