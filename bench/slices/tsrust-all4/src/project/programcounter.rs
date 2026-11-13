//! Go `internal/project/programcounter.go`.

use crate::project::prelude::*;

// Go: project/programcounter.go:9 programCounter
// PORT: `mu` is dropped (one thread). Go `map[*compiler.Program]int32` is
// keyed by the program address (PORTING "Programs and checkers"). The
// address is not used again while its entry exists: each counted snapshot
// holds the program's `Rc`. A Go nil map is an empty map.
#[derive(Default)]
pub struct ProgramCounter {
    pub refs: RefCell<FxHashMap<usize, i32>>,
}

// PORT: Go map key `*compiler.Program`.

impl ProgramCounter {
    // Go: project/programcounter.go:16 Ref
    // Ref increments the reference count for a program. If the program is not
    // yet tracked, it is added with a reference count of 1.

    // Go: project/programcounter.go:25 Deref

    // Go: project/programcounter.go:44 Len
    pub fn len(&self) -> i32 {
        self.refs.borrow().len() as i32
    }
}
