// A dependency's macros reached through its modules: a module re-export
// (`pub use core_lib::prelude;`) then globs of it, a glob of a dependency
// module that brings a submodule (`effect::typeparser`), an explicit
// `use` of a renamed re-export, and path calls.
pub use core_lib::prelude;
pub mod project {
    pub mod prelude {
        pub use crate::prelude::*;
    }
    pub mod builder {
        use crate::project::prelude::*;
        pub fn run(name: &str) -> Vec<String> {
            let mut v = args![name, helper()];
            v.extend(crate::prelude::args![1]);
            v.extend(core_lib::prelude::args![2]);
            v
        }
    }
}
pub mod effect {
    pub use core_lib::effect::*;
    pub mod ls {
        use crate::effect::typeparser::{ExportMember, member};
        pub fn runtime() -> &'static ExportMember {
            member!("runtime")
        }
    }
}
fn main() {
    println!("{:?}", project::builder::run("x"));
    println!("{}", effect::ls::runtime().0);
}
