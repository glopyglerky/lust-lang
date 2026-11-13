// expect: L0001
// rustc: the `ty` fragment parse reads `'fn` as a lifetime and reports it,
// though the template only stringifies the fragment.
macro_rules! m { ($t:ty) => { stringify!($t) } }
fn main() { println!("{}", m!(&'fn u8)); }
