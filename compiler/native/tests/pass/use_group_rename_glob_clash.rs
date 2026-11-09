// A renamed import in a use group (`Program as IncrementalProgram`) next to a glob that brings
// another `Program`, through a module path with a repeated segment (ts_goport build_task.rs).
mod prelude { pub struct Program(pub u8); }
mod program { pub struct Program; pub fn new_program() -> Program { Program } }
mod execute {
    pub mod incremental {
        pub mod incremental { pub struct Host; }
        pub mod program {
            pub struct Program { pub n: u32 }
            pub fn new_program(n: u32) -> Program { let _ = Program { n }; let _p = crate::prelude::Program(1); Program { n } }
            pub fn build_info_program() -> u32 { 7 }
            pub struct NestedEmitNow;
        }
    }
    pub mod build {
        use crate::execute::incremental::incremental::Host as IncrementalHost;
        use crate::execute::incremental::program::{
            NestedEmitNow, Program as IncrementalProgram, build_info_program,
            new_program as new_incremental_program,
        };
        use crate::prelude::*;
        use crate::program as tsprogram;
        pub enum Kind { Incremental(Box<IncrementalProgram>), Plain(tsprogram::Program) }
        pub fn run() -> u32 {
            let _h = IncrementalHost; let _e = NestedEmitNow; let _q = Program(2);
            let k = Kind::Incremental(Box::new(new_incremental_program(5)));
            let _ = Kind::Plain(tsprogram::new_program());
            match k { Kind::Incremental(p) => p.n + build_info_program(), Kind::Plain(_) => 0 }
        }
    }
}
fn main() { println!("{}", execute::build::run()); }
