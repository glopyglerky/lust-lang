// A macro-expanded macro_rules! and out-of-line module files: in scope of
// `mod child;` (lust rejects it, rustc E0659 here), after it (valid), at
// the top of the module file (lust rejects it), and gone again.
mod child;
fn main() { child::f(); println!("{}", util::helper()); }
