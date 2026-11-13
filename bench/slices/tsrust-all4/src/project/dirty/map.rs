//! Go `internal/project/dirty/map.go`.

use crate::project::dirty::prelude::*;
use std::hash::Hash;
use std::rc::Weak;

const MAP_DROPPED: &str = "dirty: Map dropped while one of its entries is in use";

// Go: project/dirty/map.go:5 MapEntry
// PORT: Go `m *Map` is a `Weak` (the map holds its dirty entries, so a
// strong pointer would be a cycle). Go stores the entry pointer itself in
// `m.dirty`, so the entry keeps a `Weak` to itself (`this`). The embedded Go
// `mapEntry` is the `map_entry` field.
pub struct MapEntry<K, V> {
    pub m: Weak<Map<K, V>>,
    pub map_entry: RefCell<MapEntryImpl<K, V>>,
    this: Weak<MapEntry<K, V>>,
}

// PORT: Go `&MapEntry[K, V]{m: m, mapEntry: ...}`.

impl<K: Eq + Hash + Clone, V: Cloneable + Clone> MapEntry<K, V> {
    // Go: project/dirty/entry.go:11 Key (promoted from mapEntry)
    pub fn key(&self) -> K {
        self.map_entry.borrow().key()
    }

    // Go: project/dirty/entry.go:15 Original (promoted from mapEntry)
    pub fn original(&self) -> Option<V> {
        self.map_entry.borrow().original()
    }

    // Go: project/dirty/entry.go:19 Value (promoted from mapEntry)
    pub fn value(&self) -> Option<V> {
        self.map_entry.borrow().value()
    }

    // Go: project/dirty/entry.go:27 Dirty (promoted from mapEntry)
    pub fn dirty(&self) -> bool {
        self.map_entry.borrow().dirty()
    }

    // Go: project/dirty/map.go:10 Change
    pub fn change(&self, apply: &mut dyn FnMut(&V)) {
        if self.map_entry.borrow().delete {
            crate::core::go_panic("tried to change a deleted entry".to_string());
        }
        if !self.map_entry.borrow().dirty {
            // PORT: Go calls Clone on a nil value; that dereferences nil.
            let value = self.map_entry.borrow().value.clone();
            let cloned = value
                .as_ref()
                .unwrap_or_else(|| crate::core::go_nil_dereference())
                .clone_();
            {
                let mut e = self.map_entry.borrow_mut();
                e.value = Some(cloned);
                e.dirty = true;
            }
            let m = self.m.upgrade().expect(MAP_DROPPED);
            m.dirty
                .borrow_mut()
                .insert(self.key(), self.this.upgrade().expect(MAP_DROPPED));
        }
        // PORT: no borrow is held while `apply` runs.
        let value = self.map_entry.borrow().value.clone();
        apply(
            value
                .as_ref()
                .unwrap_or_else(|| crate::core::go_nil_dereference()),
        );
    }

    // Go: project/dirty/map.go:22 Replace
    pub fn replace(&self, new_value: V) {
        if self.map_entry.borrow().delete {
            crate::core::go_panic("tried to change a deleted entry".to_string());
        }
        if !self.map_entry.borrow().dirty {
            self.map_entry.borrow_mut().dirty = true;
            let m = self.m.upgrade().expect(MAP_DROPPED);
            m.dirty
                .borrow_mut()
                .insert(self.key(), self.this.upgrade().expect(MAP_DROPPED));
        }
        self.map_entry.borrow_mut().value = Some(new_value);
    }

    // Go: project/dirty/map.go:33 ChangeIf
    pub fn change_if(
        &self,
        cond: &mut dyn FnMut(Option<&V>) -> bool,
        apply: &mut dyn FnMut(&V),
    ) -> bool {
        if cond(self.value().as_ref()) {
            self.change(apply);
            return true;
        }
        false
    }

    // Go: project/dirty/map.go:41 Delete
    pub fn delete(&self) {
        if !self.map_entry.borrow().dirty {
            let m = self.m.upgrade().expect(MAP_DROPPED);
            m.dirty
                .borrow_mut()
                .insert(self.key(), self.this.upgrade().expect(MAP_DROPPED));
        }
        self.map_entry.borrow_mut().delete = true;
    }

    // Go: project/dirty/map.go:48 Locked
    pub fn locked(&self, fn_: &mut dyn FnMut(&dyn Value<V>)) {
        fn_(self);
    }
}

impl<K: Eq + Hash + Clone, V: Cloneable + Clone> Value<V> for MapEntry<K, V> {
    fn value(&self) -> Option<V> {
        MapEntry::value(self)
    }

    fn original(&self) -> Option<V> {
        MapEntry::original(self)
    }

    fn dirty(&self) -> bool {
        MapEntry::dirty(self)
    }

    fn change(&self, apply: &mut dyn FnMut(&V)) {
        MapEntry::change(self, apply)
    }

    fn change_if(
        &self,
        cond: &mut dyn FnMut(Option<&V>) -> bool,
        apply: &mut dyn FnMut(&V),
    ) -> bool {
        MapEntry::change_if(self, cond, apply)
    }

    fn delete(&self) {
        MapEntry::delete(self)
    }

    fn locked(&self, fn_: &mut dyn FnMut(&dyn Value<V>)) {
        MapEntry::locked(self, fn_)
    }
}

// Go: project/dirty/map.go:52 Map
// PORT: Go `*Map` is shared and mutated, so the maps use `RefCell`. `dirty`
// keeps insertion order (PORT: Go map order is random). The base map is an
// `Rc` because a Go map is a reference: it is shared with the value it came
// from, and `finalize_shared` gives it back unchanged. Only `clear`
// replaces it. `this` is the Go pointer that new entries point back to.
pub struct Map<K, V> {
    pub base: RefCell<Rc<FxHashMap<K, V>>>,
    pub dirty: RefCell<IndexMap<K, Rc<MapEntry<K, V>>>>,
    this: Weak<Map<K, V>>,
}

// Go: project/dirty/map.go:57 NewMap
// PORT: takes an owned base map. `new_map_shared` takes a shared one.

// Go: project/dirty/map.go:57 NewMap
// PORT: Go keeps the caller's map without a copy. This form does the same
// with a shared `Rc` map.

impl<K: Eq + Hash + Clone, V: Cloneable + Clone> Map<K, V> {
    // Go: project/dirty/map.go:64 Get

    // Go: project/dirty/map.go:89 Add
    // Add sets a new entry in the dirty map without checking if it exists
    // in the base map. The entry added is considered dirty, so it should
    // be a fresh value, mutable until finalized (i.e., it will not be cloned
    // before changing if a change is made). If modifying an entry that may
    // exist in the base map, use `Change` instead.

    // Go: project/dirty/map.go:102 Change

    // Go: project/dirty/map.go:106 TryDelete

    // Go: project/dirty/map.go:118 Delete

    // Go: project/dirty/map.go:120 Range
    // PORT: the dirty entries are copied out first so `fn_` can change the
    // map. The base loop holds its own `Rc` of the base map, as Go ranges
    // over the map value it read (a `clear` in `fn_` does not change it).
    // Go keeps going with the base map after `fn_` stops the dirty loop;
    // the port does the same.

    // Go: project/dirty/map.go:144 Clear
    pub fn clear(&self) {
        *self.dirty.borrow_mut() = IndexMap::new();
        *self.base.borrow_mut() = Rc::new(FxHashMap::default());
    }

    // Go: project/dirty/map.go:149 Finalize
    // PORT: returns an owned map for callers that keep a plain `FxHashMap`.
    // When nothing changed, that is a copy of the base map.
    // `finalize_shared` returns the base map itself, as Go does.

    // Go: project/dirty/map.go:149 Finalize
    // PORT: when nothing changed, the result is the base map (an `Rc` clone),
    // as in Go. Otherwise it is a new map.
}
