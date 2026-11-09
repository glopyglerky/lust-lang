// expect: L0510
// A renamed leaf of a use group names a private glob import (rustc E0603).
mod prelude { pub fn new_program() -> u32 { 1 } }
mod program {
    use crate::prelude::*;
    pub struct Program;
    pub fn other() -> u32 { new_program() }
}
mod user {
    use crate::program::{Program as IncrementalProgram, new_program as new_incremental_program};
    pub fn f() -> u32 { let _p: Option<IncrementalProgram> = None; new_incremental_program() }
}
fn main() { println!("{} {}", user::f(), program::other()); }
