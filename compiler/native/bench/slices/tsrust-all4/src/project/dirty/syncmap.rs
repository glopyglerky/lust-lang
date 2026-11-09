//! Go `internal/project/dirty/syncmap.go`.
//!
//! PORT: one thread (see interfaces.rs). The entry mutexes are dropped.
//! The `proxyFor` logic stays literal: it also runs without a race, when two
//! entries loaded for the same key are both changed.

use crate::project::dirty::prelude::*;
use std::hash::Hash;
use std::rc::Weak;

const SYNC_MAP_DROPPED: &str = "dirty: SyncMap dropped while one of its entries is in use";

// Go: project/dirty/syncmap.go:10 lockedEntry
pub struct LockedEntry<K, V> {
    pub e: Rc<SyncMapEntry<K, V>>,
}

impl<K: Eq + Hash + Clone, V: Cloneable + Clone> LockedEntry<K, V> {
    // Go: project/dirty/syncmap.go:14 Value
    pub fn value(&self) -> Option<V> {
        self.e.value_locked()
    }

    // Go: project/dirty/syncmap.go:18 Original
    pub fn original(&self) -> Option<V> {
        self.e.map_entry.borrow().original.clone()
    }

    // Go: project/dirty/syncmap.go:22 Dirty
    pub fn dirty(&self) -> bool {
        self.e.map_entry.borrow().dirty
    }

    // Go: project/dirty/syncmap.go:26 Change

    // Go: project/dirty/syncmap.go:30 ChangeIf

    // Go: project/dirty/syncmap.go:38 Delete

    // Go: project/dirty/syncmap.go:42 Locked
    pub fn locked(&self, fn_: &mut dyn FnMut(&dyn Value<V>)) {
        fn_(self);
    }
}

impl<K: Eq + Hash + Clone, V: Cloneable + Clone> Value<V> for LockedEntry<K, V> {
    fn value(&self) -> Option<V> {
        LockedEntry::value(self)
    }

    fn original(&self) -> Option<V> {
        LockedEntry::original(self)
    }

    fn dirty(&self) -> bool {
        LockedEntry::dirty(self)
    }

    fn change(&self, apply: &mut dyn FnMut(&V)) {
        LockedEntry::change(self, apply)
    }

    fn change_if(
        &self,
        cond: &mut dyn FnMut(Option<&V>) -> bool,
        apply: &mut dyn FnMut(&V),
    ) -> bool {
        LockedEntry::change_if(self, cond, apply)
    }

    fn delete(&self) {
        LockedEntry::delete(self)
    }

    fn locked(&self, fn_: &mut dyn FnMut(&dyn Value<V>)) {
        LockedEntry::locked(self, fn_)
    }
}

// Go: project/dirty/syncmap.go:46 SyncMapEntry
// PORT: Go `m *SyncMap` is a `Weak` (the map holds its dirty entries). Go
// stores the entry pointer itself in `m.dirty`, so the entry keeps a `Weak`
// to itself (`this`). `mu` is dropped (one thread). The embedded Go
// `mapEntry` is the `map_entry` field.
pub struct SyncMapEntry<K, V> {
    pub m: Weak<SyncMap<K, V>>,
    pub map_entry: RefCell<MapEntryImpl<K, V>>,
    // proxyFor is set when this entry loses a race to become the dirty entry
    // for a value. Since two goroutines hold a reference to two entries that
    // may try to mutate the same underlying value, all mutations are routed
    // through the one that actually exists in the dirty map.
    pub proxy_for: RefCell<Option<Rc<SyncMapEntry<K, V>>>>,
    this: Weak<SyncMapEntry<K, V>>,
}

// PORT: Go `&SyncMapEntry[K, V]{m: m, mapEntry: ...}`.

impl<K: Eq + Hash + Clone, V: Cloneable + Clone> SyncMapEntry<K, V> {
    fn proxy(&self) -> Option<Rc<SyncMapEntry<K, V>>> {
        self.proxy_for.borrow().clone()
    }

    fn this_rc(&self) -> Rc<SyncMapEntry<K, V>> {
        self.this.upgrade().expect(SYNC_MAP_DROPPED)
    }

    // Go: project/dirty/entry.go:11 Key (promoted from mapEntry)
    pub fn key(&self) -> K {
        self.map_entry.borrow().key()
    }

    // Go: project/dirty/entry.go:15 Original (promoted from mapEntry)
    pub fn original(&self) -> Option<V> {
        self.map_entry.borrow().original()
    }

    // Go: project/dirty/syncmap.go:57 Value
    pub fn value(&self) -> Option<V> {
        if let Some(proxy_for) = self.proxy() {
            return proxy_for.value();
        }
        self.value_locked()
    }

    // Go: project/dirty/syncmap.go:66 valueLocked
    pub fn value_locked(&self) -> Option<V> {
        let e = self.map_entry.borrow();
        if e.delete {
            return None;
        }
        e.value.clone()
    }

    // Go: project/dirty/syncmap.go:74 Dirty
    pub fn dirty(&self) -> bool {
        if let Some(proxy_for) = self.proxy() {
            return proxy_for.dirty();
        }
        self.map_entry.borrow().dirty
    }

    // Go: project/dirty/syncmap.go:83 Locked
    pub fn locked(&self, fn_: &mut dyn FnMut(&dyn Value<V>)) {
        if let Some(proxy_for) = self.proxy() {
            proxy_for.locked(fn_);
            return;
        }
        fn_(&LockedEntry { e: self.this_rc() });
    }

    // Go: project/dirty/syncmap.go:93 Change

    // Go: project/dirty/syncmap.go:103 changeLocked

    // Go: project/dirty/syncmap.go:127 ChangeIf

    // Go: project/dirty/syncmap.go:141 Delete

    // Go: project/dirty/syncmap.go:163 deleteLocked

    // Go: project/dirty/syncmap.go:180 DeleteIf
}

impl<K: Eq + Hash + Clone, V: Cloneable + Clone> Value<V> for SyncMapEntry<K, V> {
    fn value(&self) -> Option<V> {
        SyncMapEntry::value(self)
    }

    fn original(&self) -> Option<V> {
        SyncMapEntry::original(self)
    }

    fn dirty(&self) -> bool {
        SyncMapEntry::dirty(self)
    }

    fn change(&self, apply: &mut dyn FnMut(&V)) {
        SyncMapEntry::change(self, apply)
    }

    fn change_if(
        &self,
        cond: &mut dyn FnMut(Option<&V>) -> bool,
        apply: &mut dyn FnMut(&V),
    ) -> bool {
        SyncMapEntry::change_if(self, cond, apply)
    }

    fn delete(&self) {
        SyncMapEntry::delete(self)
    }

    fn locked(&self, fn_: &mut dyn FnMut(&dyn Value<V>)) {
        SyncMapEntry::locked(self, fn_)
    }
}

// Go: project/dirty/syncmap.go:192 SyncMap
// PORT: Go `dirty collections.SyncMap` is a `RefCell<IndexMap>` (insertion
// order; PORT: Go map order is random). `base` is never written. It is an
// `Rc` because a Go map is a reference: the base map is shared with the
// value it came from, and `finalize_shared` gives it back unchanged. `this`
// is the Go pointer that new entries point back to.
pub struct SyncMap<K, V> {
    pub base: Rc<FxHashMap<K, V>>,
    pub dirty: RefCell<IndexMap<K, Rc<SyncMapEntry<K, V>>>>,
    this: Weak<SyncMap<K, V>>,
}

// Go: project/dirty/syncmap.go:197 NewSyncMap
// PORT: takes an owned base map. `new_sync_map_shared` takes a shared one.

// Go: project/dirty/syncmap.go:197 NewSyncMap
// PORT: Go keeps the caller's map without a copy. This form does the same
// with a shared `Rc` map.

impl<K: Eq + Hash + Clone, V: Cloneable + Clone> SyncMap<K, V> {
    // PORT: Go `m.dirty.LoadOrStore(key, value)` (collections.SyncMap).

    // Go: project/dirty/syncmap.go:204 Load

    // Go: project/dirty/syncmap.go:226 LoadOrStore

    // Go: project/dirty/syncmap.go:268 Delete

    // Go: project/dirty/syncmap.go:274 Range
    // PORT: the entries are copied out first so `fn_` can change the map
    // (Go sync.Map.Range may or may not see entries stored meanwhile). Go
    // keeps going with the base map after `fn_` stops the dirty Range; the
    // port does the same.

    // Go: project/dirty/syncmap.go:308 finalize
    // PORT: returns an owned map for callers that keep a plain `FxHashMap`.
    // When nothing changed, that is a copy of the base map.
    // `finalize_shared` returns the base map itself, as Go does.

    // Go: project/dirty/syncmap.go:308 finalize
    // PORT: when nothing changed, the result is the base map (an `Rc` clone),
    // as in Go. When an entry changed, it is a new map. No borrow is held
    // while a hook runs.

    // Go: project/dirty/syncmap.go:349 Finalize
    // PORT: `_exported`, because Go also has the unexported `finalize`.

    // Go: project/dirty/syncmap.go:353 FinalizeWith
}

// Go: project/dirty/syncmap.go:302 FinalizationHooks
// PORT: Go func fields; nil is `None`. Values are `Option<&V>` because Go can
// pass the zero value (nil).
pub struct FinalizationHooks<'a, K, V> {
    pub on_delete: Option<&'a mut dyn FnMut(&K, Option<&V>)>,
    pub on_change: Option<&'a mut dyn FnMut(&K, Option<&V>, Option<&V>)>,
    pub on_add: Option<&'a mut dyn FnMut(&K, Option<&V>)>,
}

impl<K, V> Default for FinalizationHooks<'_, K, V> {
    fn default() -> Self {
        FinalizationHooks {
            on_delete: None,
            on_change: None,
            on_add: None,
        }
    }
}
