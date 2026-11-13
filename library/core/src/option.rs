// `core::option`: the inherent methods of `Option<T>` (ARCH 3.1,
// W-STD-A). The enum itself is a lang type the compiler builds in;
// these impls are std source, spliced into a compile that names one
// of the methods (`STD_LANG_METHODS` in src/ty/lang.rs lists them).
//
// Signatures, bodies, and panic texts follow Rust 1.99's core, minus
// what lust has no form for yet: `const` fns and `[const]` bounds,
// `unsafe` (`unwrap_unchecked` becomes a plain match), and `Pin`.
// `#[track_caller]` sits where core has it.
//
// Not here yet (a call rejects): as_pin_ref, as_pin_mut, and
// unwrap_unchecked. `iter`,
// `iter_mut`, and `into_iter` are builtin (check/iter.rs), like the
// Vec forms.

impl<T> Option<T> {
    pub const fn is_some(&self) -> bool {
        match *self {
            Some(_) => true,
            None => false,
        }
    }

    pub fn is_some_and(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            None => false,
            Some(x) => f(x),
        }
    }

    pub const fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn is_none_or(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            None => true,
            Some(x) => f(x),
        }
    }

    pub const fn as_ref(&self) -> Option<&T> {
        match *self {
            Some(ref x) => Some(x),
            None => None,
        }
    }

    pub const fn as_mut(&mut self) -> Option<&mut T> {
        match *self {
            Some(ref mut x) => Some(x),
            None => None,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        match *self {
            Some(ref x) => core::slice::from_ref(x),
            None => &[],
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match *self {
            Some(ref mut x) => core::slice::from_mut(x),
            None => &mut [],
        }
    }

    #[track_caller]
    pub const fn expect(self, msg: &str) -> T {
        match self {
            Some(val) => val,
            // The panic formats a copy made here: a borrow of `msg` itself kept it on the
            // stack from every call's entry (core puts this panic in a cold `expect_failed`).
            None => {
                let m = msg;
                panic!("{}", m)
            }
        }
    }

    #[track_caller]
    pub const fn unwrap(self) -> T {
        match self {
            Some(val) => val,
            None => panic!("called `Option::unwrap()` on a `None` value"),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Some(x) => x,
            None => default,
        }
    }

    #[track_caller]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Some(x) => x,
            None => f(),
        }
    }

    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Some(x) => x,
            None => T::default(),
        }
    }

    pub fn map<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Some(x) => Some(f(x)),
            None => None,
        }
    }

    pub fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Some(ref x) = self {
            f(x);
        }

        self
    }

    pub fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Some(t) => f(t),
            None => default,
        }
    }

    pub fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(T) -> U,
    {
        match self {
            Some(t) => f(t),
            None => default(),
        }
    }

    pub fn map_or_default<U, F>(self, f: F) -> U
    where
        U: Default,
        F: FnOnce(T) -> U,
    {
        match self {
            Some(t) => f(t),
            None => U::default(),
        }
    }

    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Some(v) => Ok(v),
            None => Err(err),
        }
    }

    pub fn ok_or_else<E, F>(self, err: F) -> Result<T, E>
    where
        F: FnOnce() -> E,
    {
        match self {
            Some(v) => Ok(v),
            None => Err(err()),
        }
    }

    pub fn and<U>(self, optb: Option<U>) -> Option<U> {
        match self {
            Some(_) => optb,
            None => None,
        }
    }

    pub fn and_then<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> Option<U>,
    {
        match self {
            Some(x) => f(x),
            None => None,
        }
    }

    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: FnOnce(&T) -> bool,
    {
        if let Some(x) = self {
            if predicate(&x) {
                return Some(x);
            }
        }
        None
    }

    pub fn or(self, optb: Option<T>) -> Option<T> {
        match self {
            x @ Some(_) => x,
            None => optb,
        }
    }

    pub fn or_else<F>(self, f: F) -> Option<T>
    where
        F: FnOnce() -> Option<T>,
    {
        match self {
            x @ Some(_) => x,
            None => f(),
        }
    }

    pub fn xor(self, optb: Option<T>) -> Option<T> {
        match (self, optb) {
            (a @ Some(_), None) => a,
            (None, b @ Some(_)) => b,
            _ => None,
        }
    }

    pub fn insert(&mut self, value: T) -> &mut T {
        *self = Some(value);

        match self {
            Some(v) => v,
            None => unreachable!(),
        }
    }

    pub fn get_or_insert(&mut self, value: T) -> &mut T {
        self.get_or_insert_with(|| value)
    }

    pub fn get_or_insert_default(&mut self) -> &mut T
    where
        T: Default,
    {
        self.get_or_insert_with(|| T::default())
    }

    pub fn get_or_insert_with<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        if let None = self {
            *self = Some(f());
        }

        match self {
            Some(v) => v,
            None => unreachable!(),
        }
    }

    pub const fn take(&mut self) -> Option<T> {
        core::mem::replace(self, None)
    }

    pub fn take_if<P>(&mut self, predicate: P) -> Option<T>
    where
        P: FnOnce(&mut T) -> bool,
    {
        if self.as_mut().map_or(false, predicate) { self.take() } else { None }
    }

    pub const fn replace(&mut self, value: T) -> Option<T> {
        core::mem::replace(self, Some(value))
    }

    pub fn zip<U>(self, other: Option<U>) -> Option<(T, U)> {
        match (self, other) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

// `as_deref` and `as_deref_mut`, as core has them: one impl over `T:
// Deref` (`DerefMut`); `<T as Deref>::Target` is core's `T::Target`.
// On a builtin `T` the `deref` call is a conversion builtin
// (check/convert.rs `deref_call`).
impl<T: core::ops::Deref> Option<T> {
    pub fn as_deref(&self) -> Option<&<T as core::ops::Deref>::Target> {
        match self.as_ref() {
            Some(t) => Some(t.deref()),
            None => None,
        }
    }
}

impl<T: core::ops::DerefMut> Option<T> {
    pub fn as_deref_mut(&mut self) -> Option<&mut <T as core::ops::Deref>::Target> {
        match self.as_mut() {
            Some(t) => Some(t.deref_mut()),
            None => None,
        }
    }
}

impl<T, U> Option<(T, U)> {
    pub fn unzip(self) -> (Option<T>, Option<U>) {
        match self {
            Some((a, b)) => (Some(a), Some(b)),
            None => (None, None),
        }
    }
}

impl<T> Option<&T> {
    pub const fn copied(self) -> Option<T>
    where
        T: Copy,
    {
        match self {
            Some(&v) => Some(v),
            None => None,
        }
    }

    pub fn cloned(self) -> Option<T>
    where
        T: Clone,
    {
        match self {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }
}

impl<T> Option<&mut T> {
    pub const fn copied(self) -> Option<T>
    where
        T: Copy,
    {
        match self {
            Some(&mut t) => Some(t),
            None => None,
        }
    }

    pub fn cloned(self) -> Option<T>
    where
        T: Clone,
    {
        match self {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }
}

impl<T, E> Option<Result<T, E>> {
    pub const fn transpose(self) -> Result<Option<T>, E> {
        match self {
            Some(Ok(x)) => Ok(Some(x)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

impl<T> Option<Option<T>> {
    pub const fn flatten(self) -> Option<T> {
        match self {
            Some(inner) => inner,
            None => None,
        }
    }
}
