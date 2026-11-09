//! Port of Go `ast/ast.go` parts that the language service needs:
//! lines 2405-2450 (`SourceFileDataKey`, `NewSourceFileDataKey`,
//! `(*SourceFile).GetOrComputeData`, `(*SourceFile).getDataCell`), 2462-2465
//! (`TokenCacheKey`) and 2776-2838 (`(*SourceFile).GetOrCreateToken`,
//! `createToken`).
//!
//! PORT: Go keeps `data`, `tokenCache` and `tokenFactory` on each
//! `SourceFile` behind mutexes. Parsed files are immutable here, so these are
//! thread-local maps keyed by the source file `Node`, like
//! `source_file_get_declaration_map` in `ast/node.rs`. Created tokens are
//! synthetic nodes, which are thread-local too. All language-service state
//! runs on the LSP dispatch thread (PORTING "Language service", Threads).
//! The maps are keyed first by the file, so they forget a dead file version
//! with its entries (`PerFileMap`, lsshells M3a).

use crate::prelude::*;
use std::any::Any;
use std::cell::OnceCell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};


// Go: ast/ast.go:2386 SourceFileDataKey
/// SourceFileDataKey identifies lazily-computed data attached to a SourceFile by
/// another package. Prefer regular SourceFile fields for ast-owned data.
// PORT: Go `key sourceFileDataKey` is `u64` (Go `type sourceFileDataKey
// uint64`; its Rust type name would be the same as this struct's). Go
// `_ [0]T` is `PhantomData<fn() -> T>`, so a key can live in a `static` for
// any `T`. Go returns a pointer from `NewSourceFileDataKey`; here the key is
// a `Copy` value, and its `key` number is its identity.
pub struct SourceFileDataKey<T> {
    key: u64,
    _t: PhantomData<fn() -> T>,
}



impl<T> std::fmt::Debug for SourceFileDataKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceFileDataKey")
            .field("key", &self.key)
            .finish()
    }
}

// Go: ast/ast.go:2393 sourceFileDataKeyCounter
static SOURCE_FILE_DATA_KEY_COUNTER: AtomicU64 = AtomicU64::new(0);

// Go: ast/ast.go:2395 sourceFileDataCell
// PORT: Go `once sync.Once` and `value T` are one `OnceCell<T>`.
pub struct SourceFileDataCell<T> {
    value: OnceCell<T>,
}

// Go: ast/ast.go:2400 NewSourceFileDataKey
pub fn new_source_file_data_key<T>() -> SourceFileDataKey<T> {
    SourceFileDataKey {
        // Go `atomic.Uint64.Add(1)` returns the new value.
        key: SOURCE_FILE_DATA_KEY_COUNTER.fetch_add(1, Ordering::SeqCst) + 1,
        _t: PhantomData,
    }
}

// Go: ast/ast.go:2404 (*SourceFile).GetOrComputeData (ts#63902)
// PORT: Go `compute func(*SourceFile) T` is `FnOnce(Node) -> T`; the value
// is returned by clone (Go copies `T`; callers use `Rc` or `Copy` values).
// `compute` runs without any thread-local borrow held, so it can read other
// data keys of the same file.

// Go: ast/ast.go:2412 (*SourceFile).getDataCell (ts#63902)
// PORT: a Rust reference cannot be nil, so only `key.key == 0` is checked.

// Go: ast/ast.go:2441 TokenCacheKey
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TokenCacheKey {
    parent: Node,
    loc: TextRange,
}

// Go: ast/ast.go:2776 (*SourceFile).GetOrCreateToken

// Go: ast/ast.go:2935 createToken
