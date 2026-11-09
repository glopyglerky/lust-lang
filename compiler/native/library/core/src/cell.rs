// `core::cell` overlay (docs/ARCH.md section 3.1; W-STD-C): `OnceCell`
// and `LazyCell`. `Cell`, `RefCell`, and the other `cell` items stay
// compiler-known; `core::cell::X` and `std::cell::X` walk into this
// module only for the names it defines (`STD_MODULES` in
// src/stdmods.rs).
//
// Lust std source, adapted from rust-lang/rust 1.99
// `core/src/cell/once.rs` and `core/src/cell/lazy.rs`. Std keeps the
// value in an `UnsafeCell<Option<T>>`; here it is a plain `Option<T>`
// that only the privileged `core::intrinsics::once_init` and
// `once_take` builtins write through `&self`. Single-threaded semantics.
//
// Auto traits and variance as std's: the marker field makes the cells
// invariant in `T` (`fn(T) -> T`) and `!Sync` but still `Send` (a `dyn`
// of a private trait that is `Send` and not `Sync`).

/// A private trait: `&mut (dyn NotSync + Send)` is `Send` and not `Sync`.
trait NotSync {}

/// The marker every cell here carries (see the file header).
type CellMarker<T> = core::marker::PhantomData<(fn(T) -> T, &'static mut (dyn NotSync + Send))>;

/// Debug text of an empty cell: `<uninit>`, as std's
/// `format_args!("<uninit>")` field prints.
struct Uninit;

impl core::fmt::Debug for Uninit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("<uninit>")
    }
}

/// A cell that can be written to only once.
pub struct OnceCell<T> {
    inner: Option<T>,
    _marker: CellMarker<T>,
}

impl<T> OnceCell<T> {
    pub const fn new() -> OnceCell<T> {
        OnceCell { inner: None, _marker: core::marker::PhantomData }
    }

    pub fn get(&self) -> Option<&T> {
        match &self.inner {
            Some(v) => Some(v),
            None => None,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match &mut self.inner {
            Some(v) => Some(v),
            None => None,
        }
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        if self.inner.is_some() {
            return Err(value);
        }
        core::intrinsics::once_init(&self.inner, value);
        Ok(())
    }

    pub fn get_or_init<F: FnOnce() -> T>(&self, f: F) -> &T {
        if self.inner.is_none() {
            let value = f();
            if self.inner.is_some() {
                panic!("reentrant init"); //@ core/src/cell/once.rs:325:66
            }
            core::intrinsics::once_init(&self.inner, value);
        }
        match &self.inner {
            Some(v) => v,
            None => unreachable!(), //@ never
        }
    }

    /// Std's is `const fn` (it leans on const_precise_live_drops); lust
    /// would see the cell's drop in a const fn, so this one is not const
    /// (a const caller is a false reject, never a false accept).
    pub fn into_inner(self) -> Option<T> {
        self.inner
    }

    pub fn take(&mut self) -> Option<T> {
        core::mem::replace(&mut self.inner, None)
    }
}

impl<T> Default for OnceCell<T> {
    fn default() -> OnceCell<T> {
        OnceCell::new()
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for OnceCell<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_tuple("OnceCell");
        match &self.inner {
            Some(v) => d.field(v),
            None => d.field(&Uninit),
        };
        d.finish()
    }
}

impl<T: Clone> Clone for OnceCell<T> {
    fn clone(&self) -> OnceCell<T> {
        match &self.inner {
            Some(v) => OnceCell { inner: Some(v.clone()), _marker: core::marker::PhantomData },
            None => OnceCell::new(),
        }
    }
}

impl<T: PartialEq> PartialEq for OnceCell<T> {
    fn eq(&self, other: &OnceCell<T>) -> bool {
        match (&self.inner, &other.inner) {
            (Some(a), Some(b)) => *a == *b,
            (None, None) => true,
            _ => false,
        }
    }
}

impl<T: Eq> Eq for OnceCell<T> {}

impl<T> From<T> for OnceCell<T> {
    fn from(value: T) -> OnceCell<T> {
        OnceCell { inner: Some(value), _marker: core::marker::PhantomData }
    }
}

/// A value initialized on first access by `F`.
pub struct LazyCell<T, F = fn() -> T> {
    cell: OnceCell<T>,
    init: Option<F>,
}

impl<T, F: FnOnce() -> T> LazyCell<T, F> {
    pub const fn new(f: F) -> LazyCell<T, F> {
        LazyCell { cell: OnceCell::new(), init: Some(f) }
    }

    pub fn force(this: &LazyCell<T, F>) -> &T {
        if this.cell.inner.is_none() {
            match core::intrinsics::once_take(&this.init) {
                Some(f) => {
                    let value = f();
                    core::intrinsics::once_init(&this.cell.inner, value);
                }
                None => panic!("LazyCell instance has previously been poisoned"), //@ core/src/cell/lazy.rs:383:5
            }
        }
        match &this.cell.inner {
            Some(v) => v,
            None => unreachable!(), //@ never
        }
    }

    pub fn force_mut(this: &mut LazyCell<T, F>) -> &mut T {
        if this.cell.inner.is_none() {
            match core::intrinsics::once_take(&this.init) {
                Some(f) => {
                    let value = f();
                    core::intrinsics::once_init(&this.cell.inner, value);
                }
                None => panic!("LazyCell instance has previously been poisoned"), //@ core/src/cell/lazy.rs:383:5
            }
        }
        match &mut this.cell.inner {
            Some(v) => v,
            None => unreachable!(), //@ never
        }
    }

    pub fn get(this: &LazyCell<T, F>) -> Option<&T> {
        this.cell.get()
    }

    pub fn get_mut(this: &mut LazyCell<T, F>) -> Option<&mut T> {
        this.cell.get_mut()
    }
}

impl<T, F: FnOnce() -> T> core::ops::Deref for LazyCell<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        LazyCell::force(self)
    }
}

impl<T, F> From<T> for LazyCell<T, F> {
    fn from(value: T) -> LazyCell<T, F> {
        LazyCell { cell: OnceCell { inner: Some(value), _marker: core::marker::PhantomData }, init: None }
    }
}

impl<T: core::fmt::Debug, F> core::fmt::Debug for LazyCell<T, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_tuple("LazyCell");
        match &self.cell.inner {
            Some(v) => d.field(v),
            None => d.field(&Uninit),
        };
        d.finish()
    }
}
