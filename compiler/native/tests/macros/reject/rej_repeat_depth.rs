// reject: transcription: a metavariable repeats at a depth it was not matched at (ARCH 6.2).
// rustc-error: attempted to repeat an expression containing no syntax variables
macro_rules! m { ($x:ident) => { $($x)* } }
fn main() { m!(a); }
