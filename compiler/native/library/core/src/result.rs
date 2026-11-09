// `core::result`: the inherent methods of `Result<T, E>` (ARCH 3.1,
// W-STD-A). The enum itself is a lang type the compiler builds in;
// see option.rs in this crate for how the splice works and what is
// left out of core's forms.
//
// Not here yet (a call rejects): unwrap_unchecked and
// unwrap_err_unchecked.
// `iter`, `iter_mut`, and `into_iter` are builtin (check/iter.rs).

impl<T, E> Result<T, E> {
    pub const fn is_ok(&self) -> bool {
        match *self {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn is_ok_and<F>(self, f: F) -> bool
    where
        F: FnOnce(T) -> bool,
    {
        match self {
            Err(_) => false,
            Ok(x) => f(x),
        }
    }

    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    pub fn is_err_and<F>(self, f: F) -> bool
    where
        F: FnOnce(E) -> bool,
    {
        match self {
            Ok(_) => false,
            Err(e) => f(e),
        }
    }

    pub fn ok(self) -> Option<T> {
        match self {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }

    pub fn err(self) -> Option<E> {
        match self {
            Ok(_) => None,
            Err(x) => Some(x),
        }
    }

    pub const fn as_ref(&self) -> Result<&T, &E> {
        match *self {
            Ok(ref x) => Ok(x),
            Err(ref x) => Err(x),
        }
    }

    pub const fn as_mut(&mut self) -> Result<&mut T, &mut E> {
        match *self {
            Ok(ref mut x) => Ok(x),
            Err(ref mut x) => Err(x),
        }
    }

    pub fn map<U, F>(self, op: F) -> Result<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Ok(t) => Ok(op(t)),
            Err(e) => Err(e),
        }
    }

    pub fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Ok(t) => f(t),
            Err(_) => default,
        }
    }

    pub fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce(E) -> U,
        F: FnOnce(T) -> U,
    {
        match self {
            Ok(t) => f(t),
            Err(e) => default(e),
        }
    }

    pub fn map_or_default<U, F>(self, f: F) -> U
    where
        F: FnOnce(T) -> U,
        U: Default,
    {
        match self {
            Ok(t) => f(t),
            Err(_) => U::default(),
        }
    }

    pub fn map_err<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(E) -> F,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(op(e)),
        }
    }

    pub fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Ok(ref t) = self {
            f(t);
        }

        self
    }

    pub fn inspect_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&E),
    {
        if let Err(ref e) = self {
            f(e);
        }

        self
    }

    #[track_caller]
    pub fn expect(self, msg: &str) -> T
    where
        E: core::fmt::Debug,
    {
        match self {
            Ok(t) => t,
            // The panic formats a copy made here: a borrow of `msg` itself kept it on the
            // stack from every call's entry (core puts this panic in a cold `unwrap_failed`).
            Err(e) => {
                let m = msg;
                panic!("{}: {:?}", m, e)
            }
        }
    }

    #[track_caller]
    pub fn unwrap(self) -> T
    where
        E: core::fmt::Debug,
    {
        match self {
            Ok(t) => t,
            Err(e) => panic!("called `Result::unwrap()` on an `Err` value: {:?}", e),
        }
    }

    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Ok(x) => x,
            Err(_) => T::default(),
        }
    }

    #[track_caller]
    pub fn expect_err(self, msg: &str) -> E
    where
        T: core::fmt::Debug,
    {
        match self {
            // A copy, as in `expect`.
            Ok(t) => {
                let m = msg;
                panic!("{}: {:?}", m, t)
            }
            Err(e) => e,
        }
    }

    #[track_caller]
    pub fn unwrap_err(self) -> E
    where
        T: core::fmt::Debug,
    {
        match self {
            Ok(t) => panic!("called `Result::unwrap_err()` on an `Ok` value: {:?}", t),
            Err(e) => e,
        }
    }

    pub fn and<U>(self, res: Result<U, E>) -> Result<U, E> {
        match self {
            Ok(_) => res,
            Err(e) => Err(e),
        }
    }

    pub fn and_then<U, F>(self, op: F) -> Result<U, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        match self {
            Ok(t) => op(t),
            Err(e) => Err(e),
        }
    }

    pub fn or<F>(self, res: Result<T, F>) -> Result<T, F> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => res,
        }
    }

    pub fn or_else<F, O>(self, op: O) -> Result<T, F>
    where
        O: FnOnce(E) -> Result<T, F>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => op(e),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Ok(t) => t,
            Err(_) => default,
        }
    }

    #[track_caller]
    pub fn unwrap_or_else<F>(self, op: F) -> T
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Ok(t) => t,
            Err(e) => op(e),
        }
    }
}

// `as_deref` and `as_deref_mut`, as core has them: one impl over `T:
// Deref` (`DerefMut`); see option.rs.
impl<T: core::ops::Deref, E> Result<T, E> {
    pub fn as_deref(&self) -> Result<&<T as core::ops::Deref>::Target, &E> {
        self.as_ref().map(|t| t.deref())
    }
}

impl<T: core::ops::DerefMut, E> Result<T, E> {
    pub fn as_deref_mut(&mut self) -> Result<&mut <T as core::ops::Deref>::Target, &mut E> {
        self.as_mut().map(|t| t.deref_mut())
    }
}

impl<T, E> Result<&T, E> {
    pub const fn copied(self) -> Result<T, E>
    where
        T: Copy,
    {
        match self {
            Ok(&v) => Ok(v),
            Err(e) => Err(e),
        }
    }

    pub fn cloned(self) -> Result<T, E>
    where
        T: Clone,
    {
        self.map(|t| t.clone())
    }
}

impl<T, E> Result<&mut T, E> {
    pub const fn copied(self) -> Result<T, E>
    where
        T: Copy,
    {
        match self {
            Ok(&mut v) => Ok(v),
            Err(e) => Err(e),
        }
    }

    pub fn cloned(self) -> Result<T, E>
    where
        T: Clone,
    {
        self.map(|t| t.clone())
    }
}

impl<T, E> Result<Option<T>, E> {
    pub const fn transpose(self) -> Option<Result<T, E>> {
        match self {
            Ok(Some(x)) => Some(Ok(x)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<T, E> Result<Result<T, E>, E> {
    pub const fn flatten(self) -> Result<T, E> {
        match self {
            Ok(inner) => inner,
            Err(e) => Err(e),
        }
    }
}
