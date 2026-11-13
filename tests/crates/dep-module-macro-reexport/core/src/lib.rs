// Macros re-exported in this crate's modules, as ts-rust's goport_core does
// (`prelude::args`, `effect::typeparser::member`).
pub mod util {
    #[macro_export]
    macro_rules! args {
        ($($arg:expr),* $(,)?) => { vec![$(::std::string::ToString::to_string(&$arg)),*] };
    }
    pub fn helper() -> u32 {
        7
    }
}
pub mod prelude {
    pub use crate::args;
    pub use crate::util::*;
}
pub mod effect {
    pub mod typeparser {
        mod export_ref {
            pub struct ExportMember(pub &'static str);
            impl ExportMember {
                pub const fn new(s: &'static str) -> Self {
                    ExportMember(s)
                }
            }
            #[macro_export]
            #[doc(hidden)]
            macro_rules! effect_export_member {
                ($name:literal) => {{
                    static MEMBER: $crate::effect::typeparser::ExportMember = $crate::effect::typeparser::ExportMember::new($name);
                    &MEMBER
                }};
            }
            pub use crate::effect_export_member as member;
        }
        pub use export_ref::*;
    }
}
