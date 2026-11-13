// K1 compatibility fixture: ArenaBox with Deref, DerefMut, Clone, Debug.
// panic-mode: abort
//
// Real shape: crates/ts_goport/src/checker/types.rs ArenaBox<T>
// (enum with &'static mut T arena repr and Box owned repr), copied
// with one replacement: checker_arena().alloc_with (a bump arena)
// becomes Box::leak, which has the same observable rule for this
// fixture (an arena value is never dropped). A flag picks the repr
// per call so both run; ts-rust picks once per process.

use std::cell::Cell;

std::thread_local! {
    static USE_ARENA: Cell<bool> = const { Cell::new(true) };
    static DROPS: Cell<u32> = const { Cell::new(0) };
}

fn use_checker_arena() -> bool {
    USE_ARENA.with(|f| f.get())
}

/// Stand-in for the checker bump arena slot maker.
fn arena_alloc_with<T: 'static>(make: impl FnOnce() -> T) -> &'static mut T {
    Box::leak(Box::new(make()))
}

/// A value in this thread's checker arena, or a Box. Verbatim from
/// ts-rust checker/types.rs except the arena slot maker above.
pub struct ArenaBox<T: 'static>(ArenaBoxRepr<T>);

enum ArenaBoxRepr<T: 'static> {
    Arena(&'static mut T),
    Owned(Box<T>),
}

impl<T> ArenaBox<T> {
    #[inline(always)]
    pub fn new(value: T) -> Self {
        Self::new_with(move || value)
    }

    #[inline(always)]
    pub fn new_with(make: impl FnOnce() -> T) -> Self {
        ArenaBox(if use_checker_arena() {
            ArenaBoxRepr::Arena(arena_alloc_with(make))
        } else {
            ArenaBoxRepr::Owned(Box::write(Box::new_uninit(), make()))
        })
    }
}

impl<T> std::ops::Deref for ArenaBox<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        match &self.0 {
            ArenaBoxRepr::Arena(value) => value,
            ArenaBoxRepr::Owned(value) => value,
        }
    }
}

impl<T> std::ops::DerefMut for ArenaBox<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        match &mut self.0 {
            ArenaBoxRepr::Arena(value) => value,
            ArenaBoxRepr::Owned(value) => value,
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ArenaBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

/// A clone is a new copy, like a Box clone.
impl<T: Clone> Clone for ArenaBox<T> {
    fn clone(&self) -> Self {
        ArenaBox::new((**self).clone())
    }
}

/// A value that records its drop, like a union type payload would
/// through its Vec and String fields.
#[derive(Clone, Debug)]
struct Payload {
    name: String,
    items: Vec<i32>,
}

impl Payload {
    fn total(&self) -> i32 {
        self.items.iter().sum()
    }
}

impl Drop for Payload {
    fn drop(&mut self) {
        DROPS.with(|d| d.set(d.get() + 1));
    }
}

fn drops() -> u32 {
    DROPS.with(|d| d.get())
}

fn main() {
    // Arena repr: method calls and field reads go through Deref,
    // writes through DerefMut. Dropping the box drops no value.
    USE_ARENA.with(|f| f.set(true));
    {
        let mut a = ArenaBox::new(Payload {
            name: "union".to_string(),
            items: vec![1, 2, 3],
        });
        println!("arena name={} total={}", a.name, a.total());
        a.items.push(4);
        a.name.push_str("-grown");
        println!("arena after mut: {:?}", a);
        let b = a.clone();
        println!("arena clone total={}", b.total());
    }
    println!("drops after arena scope: {}", drops()); // 0: leaked like an arena

    // Owned repr: same API, values drop with the box.
    USE_ARENA.with(|f| f.set(false));
    {
        let a = ArenaBox::new_with(|| Payload {
            name: "owned".to_string(),
            items: vec![10, 20],
        });
        println!("owned name={} total={}", a.name, a.total());
        let b = a.clone();
        println!("owned clone: {:?}", b);
        // a and b drop at the end of this scope.
    }
    println!("drops after owned scope: {}", drops());
}
