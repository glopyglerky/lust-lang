// reject: a macro definition not delimited with braces must end with `;` (ARCH 6.2).
// rustc-error: macros that expand to items must be delimited with braces or followed by a semicolon
macro_rules! m ( () => { 1 } )
fn main() { let _ = m!(); }
