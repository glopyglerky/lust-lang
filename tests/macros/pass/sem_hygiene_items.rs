// semantics: item paths in a macro body resolve at the call site
// (ARCH 6.3, review arch-r2-sol B). An unqualified helper() in the
// macro finds the invoking module's helper, not the defining
// module's. Item names a macro defines land in the caller's module.
// $crate is the only def-site path form.

pub mod hidden {
    pub fn secret_value() -> i32 {
        77
    }
}

#[macro_use]
pub mod defs {
    pub fn helper() -> i32 {
        1
    }
    macro_rules! call_helper {
        () => {
            helper()
        };
    }
    macro_rules! make_item {
        () => {
            pub fn generated() -> i32 {
                55
            }
        };
    }
    pub(crate) use call_helper;
    pub(crate) use make_item;
}

#[macro_export]
macro_rules! crate_secret {
    () => {
        $crate::hidden::secret_value()
    };
}

pub mod caller {
    pub fn helper() -> i32 {
        2
    }
    crate::defs::make_item!();
    pub fn run() -> (i32, i32, i32) {
        // helper() inside the macro resolves here and returns 2.
        // generated() was defined into this module by the macro.
        // $crate reaches the crate root module from any call site.
        (crate::defs::call_helper!(), generated(), crate::crate_secret!())
    }
}

fn main() {
    let (h, g, s) = caller::run();
    println!("call-site helper={h} generated item={g} crate secret={s}");
    println!("def-site helper still={}", defs::helper());
}
