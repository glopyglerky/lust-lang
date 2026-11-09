// reject: an empty expansion in expression position ends with an incomplete expression (ARCH 6.2).
// rustc-error: macro expansion ends with an incomplete expression
macro_rules! nothing { () => {}; }
fn main() { let _y: () = nothing!(); }
