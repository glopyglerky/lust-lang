// `std::panic` overlay (docs/ARCH.md section 7.2; W-UNWIND):
// `AssertUnwindSafe`, `PanicHookInfo`, `set_hook`, and `take_hook` (the
// hook lives in the runtime; the runtime calls `__lust_run_hook` at a
// panic). `catch_unwind`, `resume_unwind`, and `panic_any`
// stay compiler-known (check/unwind.rs); `std::panic::X` walks into this
// module only for the names it defines (`STD_MODULES` in src/stdmods.rs).
//
// Lust std source, adapted from rust-lang/rust 1.99
// `core/src/panic/unwind_safe.rs`. Not modeled here: the `UnwindSafe` and
// `RefUnwindSafe` impls (the checker keeps that table), `FnOnce` for
// `AssertUnwindSafe<F>` (the checker forwards it to `F`), and the Debug,
// Default, and Future impls.

/// A wrapper that asserts a value is unwind safe.
pub struct AssertUnwindSafe<T>(pub T);

impl<T> core::ops::Deref for AssertUnwindSafe<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> core::ops::DerefMut for AssertUnwindSafe<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// What a panic hook receives: the payload and where the panic started.
/// Both live in the runtime for the hook call (runtime/panic.c
/// `lust_run_hook`) and the accessors read them there: lust's region
/// model ties a stored reference to the struct's lifetime.
pub struct PanicHookInfo<'a> {
    can_unwind: bool,
    _life: core::marker::PhantomData<&'a ()>,
}

impl<'a> PanicHookInfo<'a> {
    pub fn payload(&self) -> &(dyn core::any::Any + Send) {
        core::intrinsics::panic_hook_payload()
    }

    pub fn payload_as_str(&self) -> Option<&str> {
        let payload = self.payload();
        if let Some(s) = payload.downcast_ref::<&str>() {
            Some(*s)
        } else if let Some(s) = payload.downcast_ref::<String>() {
            Some(s.as_str())
        } else {
            None
        }
    }

    pub fn location(&self) -> Option<&core::panic::Location<'_>> {
        Some(core::intrinsics::panic_hook_location())
    }

    pub fn can_unwind(&self) -> bool {
        self.can_unwind
    }
}

impl core::fmt::Display for PanicHookInfo<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("panicked at ")?;
        write!(f, "{}", core::intrinsics::panic_hook_location())?;
        if let Some(payload) = self.payload_as_str() {
            f.write_str(":\n")?;
            f.write_str(payload)?;
        }
        Ok(())
    }
}

/// The installed hook. It lives in the runtime (runtime/panic.c
/// `lust_hook_slot`, all zero bytes: None, std's default hook), where
/// the runtime's panic path reads it through `__lust_run_hook`.
type Hook = Option<Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static>>;

/// Installs `hook`, dropping the one before it.
pub fn set_hook(hook: Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static>) {
    if std::thread::panicking() {
        panic!("cannot modify the panic hook from a panicking thread"); //@ std/src/panicking.rs:144:9
    }
    let old: Hook = core::intrinsics::panic_hook_swap(Some(hook));
    core::intrinsics::panic_hook_custom(true);
    drop(old);
}

/// Removes the installed hook and returns it (std's default hook when
/// none was installed).
pub fn take_hook() -> Box<dyn Fn(&PanicHookInfo<'_>) + Sync + Send + 'static> {
    if std::thread::panicking() {
        panic!("cannot modify the panic hook from a panicking thread"); //@ std/src/panicking.rs:184:9
    }
    let old: Hook = core::intrinsics::panic_hook_swap(None);
    core::intrinsics::panic_hook_custom(false);
    match old {
        Some(h) => h,
        None => Box::new(default_hook),
    }
}

fn default_hook(info: &PanicHookInfo<'_>) {
    let loc = core::intrinsics::panic_hook_location();
    core::intrinsics::panic_default_hook(loc.file(), loc.line(), loc.column(), info.payload_as_str().unwrap_or("Box<dyn Any>"));
}

/// The runtime's call into the installed hook at a panic (runtime/panic.c
/// `lust_run_hook`; main registers this fn's slot).
pub fn __lust_run_hook(can_unwind: bool) {
    let info = PanicHookInfo { can_unwind, _life: core::marker::PhantomData };
    let hook: &Hook = core::intrinsics::panic_hook_ref();
    match hook {
        Some(h) => h(&info),
        None => default_hook(&info),
    }
}

/// A type that is safe to use across a `catch_unwind` boundary. An auto
/// trait in std: lust answers its bounds from a fixed table
/// (check/unwind.rs), so it has no impls here.
pub trait UnwindSafe {}

/// A type whose shared references are `UnwindSafe` (see `UnwindSafe`).
pub trait RefUnwindSafe {}
