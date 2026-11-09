//! Go `internal/compiler/program.go` lines 1 to 444: program options, the
//! `Program` struct, construction, update and the simple accessors.
//! The `Program` type is `NewProgram` (the loader contract name), so it does
//! not clash with the program state in program.rs.

use crate::contentmapper::{Mapper, };
use crate::frontend::prelude::*;
use std::cell::OnceCell;



/// Go `lazyValue[T]`.
// PORT: Go `sync.Once` plus `initialized` is a `OnceCell` (single thread).
// Go stores a `*T` and shares it in `tryReuse`; Rust clones the value.
pub struct LazyValue<T> {
    value: OnceCell<T>,
}

impl<T> Default for LazyValue<T> {
    fn default() -> Self {
        Self {
            value: OnceCell::new(),
        }
    }
}

impl<T: Clone> LazyValue<T> {
    // Go: program.go:62 (*lazyValue[T]).getValue
    pub fn get_value(&self, compute: impl FnOnce() -> T) -> &T {
        self.value.get_or_init(compute)
    }

    /// The value when it is computed, else None (Go `initialized.Load()`,
    /// as `tryReuse` reads it).
    pub fn get(&self) -> Option<&T> {
        self.value.get()
    }

    // Go: program.go:72 (*lazyValue[T]).tryReuse
    pub fn try_reuse(&mut self, from: &LazyValue<T>) {
        if let Some(value) = from.value.get() {
            self.value = OnceCell::from(value.clone());
        }
    }
}

// Go: fileloader.go:78 (*redirectsFile).FileName, fileloader.go:82 (*redirectsFile).Path
// PORT: Go `*redirectsFile` implements `ast.HasFileName`. The inherent
// methods are in file_loader.rs; no other unit adds the trait impl.
impl HasFileName for RedirectsFile {
    fn file_name(&self) -> String {
        RedirectsFile::file_name(self)
    }
    fn path(&self) -> Path {
        RedirectsFile::path(self)
    }
}





// Go: program.go:285 NewProgram



/// Go `slices.EqualFunc`.
fn slices_equal_func<T>(s1: &[T], s2: &[T], eq: impl Fn(&T, &T) -> bool) -> bool {
    s1.len() == s2.len() && s1.iter().zip(s2).all(|(v1, v2)| eq(v1, v2))
}


// Go: program.go:495 equalModuleSpecifiers

// Go: program.go:499 equalModuleAugmentationNames

// Go: program.go:503 equalFileReferences
pub fn equal_file_references(f1: &FileReference, f2: &FileReference) -> bool {
    f1.file_name == f2.file_name
        && f1.resolution_mode == f2.resolution_mode
        && f1.preserve == f2.preserve
}

// Go: program.go:507 equalCheckJSDirectives
pub fn equal_check_js_directives(
    d1: Option<&CheckJsDirective>,
    d2: Option<&CheckJsDirective>,
) -> bool {
    match (d1, d2) {
        (None, None) => true,
        (Some(d1), Some(d2)) => d1.enabled == d2.enabled,
        _ => false,
    }
}


// PORT: the body of Go `(*Program).SingleThreaded`. It reads only `opts`, so
// `new_program` can call it before the program exists.
