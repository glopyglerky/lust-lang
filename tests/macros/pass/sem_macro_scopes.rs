// semantics: textual macro scope, #[macro_export] path scope, and
// use-imported macros (ARCH 6.1).
// - Textual: usable after the definition in the same module and in
//   child modules declared after it.
// - Path: macro_export puts the macro at the crate root. It is
//   usable as crate::name! even before the definition point and
//   importable with use.

pub mod early {
    // This module is before every definition below. The path form
    // and the use import still work.
    use crate::exported_val;
    pub fn run() -> (i32, i32) {
        (crate::exported_val!(), exported_val!() + 1)
    }
}

macro_rules! textual_val {
    () => {
        10
    };
}

pub mod child_after {
    // Declared after textual_val, so the textual macro is in scope.
    pub fn run() -> i32 {
        textual_val!() + 1
    }
}

#[macro_export]
macro_rules! exported_val {
    () => {
        20
    };
}

// The inline-module re-import shape ts-rust uses in lib.rs for
// go_flags and go_enum: pub(crate) use of a macro_export macro.
pub mod reimport {
    pub(crate) use crate::exported_val as renamed_val;
    pub fn run() -> i32 {
        renamed_val!() + 2
    }
}

fn main() {
    println!("textual={} child={}", textual_val!(), child_after::run());
    let (a, b) = early::run();
    println!("early path={a} early use={b} reimport={}", reimport::run());
}
