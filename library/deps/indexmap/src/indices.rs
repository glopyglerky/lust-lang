// The hash index of an IndexMap: hashbrown 0.17.1's `HashTable<usize>`
// as plain source. Slots hold entry indices; the caller hashes and
// compares (it keeps each entry's hash, so growth never hashes a key
// again). Probe order, control bytes, growth, tombstones, and rehash in
// place follow hashbrown, so `capacity()` and the order in which `eq`
// sees candidates are upstream's for the same hashes. The group scan is
// a byte loop; only the group width is observable (16 on x86_64 with
// SSE2, 8 for hashbrown's other groups).

#[cfg(target_arch = "x86_64")]
const WIDTH: usize = 16;
#[cfg(not(target_arch = "x86_64"))]
const WIDTH: usize = 8;

const EMPTY: u8 = 0xFF;
const DELETED: u8 = 0x80;

/// One hashbrown table of entry indices.
#[derive(Clone)]
pub(crate) struct Indices {
    /// `buckets + WIDTH` control bytes; the tail mirrors the head. The
    /// empty singleton is `WIDTH` EMPTY bytes with `bucket_mask` 0.
    ctrl: Vec<u8>,
    /// One entry index per bucket (empty for the singleton).
    slots: Vec<usize>,
    bucket_mask: usize,
    items: usize,
    growth_left: usize,
}

/// The tag of a hash: its top 7 bits.
fn tag(hash: u64) -> u8 {
    ((hash >> 57) & 0x7F) as u8
}

fn is_full(c: u8) -> bool {
    c & 0x80 == 0
}

fn bucket_mask_to_capacity(bucket_mask: usize) -> usize {
    if bucket_mask < 8 { bucket_mask } else { ((bucket_mask + 1) / 8) * 7 }
}

/// hashbrown's capacity_to_buckets for 8-byte slots; None on overflow.
fn capacity_to_buckets(cap: usize) -> Option<usize> {
    if cap < 15 {
        let cap = if cap < 3 { 3 } else { cap };
        return Some(if cap < 4 {
            4
        } else if cap < 8 {
            8
        } else {
            16
        });
    }
    let adjusted = cap.checked_mul(8)? / 7;
    let mut buckets: usize = 1;
    while buckets < adjusted {
        buckets = buckets.checked_mul(2)?;
    }
    Some(buckets)
}

impl Indices {
    /// The empty table (no allocation in hashbrown; one here, of `WIDTH` bytes).
    pub(crate) fn new() -> Indices {
        Indices { ctrl: vec![EMPTY; WIDTH], slots: Vec::new(), bucket_mask: 0, items: 0, growth_left: 0 }
    }

    /// A table that holds `capacity` items without growing.
    pub(crate) fn with_capacity(capacity: usize) -> Indices {
        if capacity == 0 {
            return Indices::new();
        }
        let buckets = match capacity_to_buckets(capacity) {
            Some(b) => b,
            None => panic!("Hash table capacity overflow"),
        };
        Indices {
            ctrl: vec![EMPTY; buckets + WIDTH],
            slots: vec![0; buckets],
            bucket_mask: buckets - 1,
            items: 0,
            growth_left: bucket_mask_to_capacity(buckets - 1),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.items
    }

    pub(crate) fn capacity(&self) -> usize {
        self.items + self.growth_left
    }

    fn num_buckets(&self) -> usize {
        self.bucket_mask + 1
    }

    fn is_singleton(&self) -> bool {
        self.bucket_mask == 0
    }

    /// The entry index in a full bucket.
    pub(crate) fn get(&self, bucket: usize) -> usize {
        self.slots[bucket]
    }

    /// Replaces the entry index in a full bucket.
    pub(crate) fn set(&mut self, bucket: usize, value: usize) {
        self.slots[bucket] = value;
    }

    /// Sets a control byte and its mirror in the tail.
    fn set_ctrl(&mut self, index: usize, c: u8) {
        let index2 = (index.wrapping_sub(WIDTH) & self.bucket_mask) + WIDTH;
        self.ctrl[index] = c;
        self.ctrl[index2] = c;
    }

    /// The group of `WIDTH` control bytes at `pos`. Every probe position
    /// is masked by `bucket_mask`, and the control bytes are `buckets +
    /// WIDTH` long, so the group is always in range: one range check per
    /// group. The scans loop to `g.len()`, so each byte's check is the
    /// loop condition the C compiler already proves (indexing `ctrl` at
    /// `pos + i` checked all `WIDTH`; hashbrown loads through a pointer).
    fn group(&self, pos: usize) -> &[u8] {
        debug_assert!(pos <= self.bucket_mask && pos + WIDTH <= self.ctrl.len());
        &self.ctrl[pos..pos + WIDTH]
    }

    /// Bit i set when control byte `pos + i` is `c`.
    fn match_byte(&self, pos: usize, c: u8) -> u32 {
        let g = self.group(pos);
        let mut bits: u32 = 0;
        let mut i = 0;
        while i < g.len() {
            if g[i] == c {
                bits |= 1 << i;
            }
            i += 1;
        }
        bits
    }

    /// Bit i set when control byte `pos + i` is EMPTY or DELETED.
    fn match_empty_or_deleted(&self, pos: usize) -> u32 {
        let g = self.group(pos);
        let mut bits: u32 = 0;
        let mut i = 0;
        while i < g.len() {
            if g[i] & 0x80 != 0 {
                bits |= 1 << i;
            }
            i += 1;
        }
        bits
    }

    /// hashbrown's fix_insert_index: a table smaller than a group can
    /// return a full mirror byte; take the first empty slot of group 0.
    fn fix_insert_index(&self, index: usize) -> usize {
        if is_full(self.ctrl[index]) { self.match_empty_or_deleted(0).trailing_zeros() as usize } else { index }
    }

    /// The first EMPTY or DELETED bucket on the probe sequence.
    fn find_insert_index(&self, hash: u64) -> usize {
        let mut pos = (hash as usize) & self.bucket_mask;
        let mut stride = 0;
        loop {
            let eod = self.match_empty_or_deleted(pos);
            if eod != 0 {
                let index = (pos + eod.trailing_zeros() as usize) & self.bucket_mask;
                return self.fix_insert_index(index);
            }
            stride += WIDTH;
            pos = (pos + stride) & self.bucket_mask;
        }
    }

    /// The bucket whose entry index satisfies `eq`, probing as hashbrown.
    pub(crate) fn find<F: FnMut(usize) -> bool>(&self, hash: u64, mut eq: F) -> Option<usize> {
        let t = tag(hash);
        let mut pos = (hash as usize) & self.bucket_mask;
        let mut stride = 0;
        loop {
            let mut m = self.match_byte(pos, t);
            while m != 0 {
                let index = (pos + m.trailing_zeros() as usize) & self.bucket_mask;
                m &= m - 1;
                if eq(self.slots[index]) {
                    return Some(index);
                }
            }
            if self.match_byte(pos, EMPTY) != 0 {
                return None;
            }
            stride += WIDTH;
            pos = (pos + stride) & self.bucket_mask;
        }
    }

    /// HashTable::entry: reserves room for one more item first (also when
    /// the key is there), then returns Ok(full bucket) or Err(insert
    /// bucket for `insert_at`).
    pub(crate) fn entry<F: FnMut(usize) -> bool, H: Fn(usize) -> u64>(&mut self, hash: u64, mut eq: F, hasher: H) -> Result<usize, usize> {
        self.reserve(1, hasher);
        let t = tag(hash);
        let mut pos = (hash as usize) & self.bucket_mask;
        let mut stride = 0;
        let mut insert: Option<usize> = None;
        loop {
            let mut m = self.match_byte(pos, t);
            while m != 0 {
                let index = (pos + m.trailing_zeros() as usize) & self.bucket_mask;
                m &= m - 1;
                if eq(self.slots[index]) {
                    return Ok(index);
                }
            }
            if insert.is_none() {
                let eod = self.match_empty_or_deleted(pos);
                if eod != 0 {
                    insert = Some((pos + eod.trailing_zeros() as usize) & self.bucket_mask);
                }
            }
            if let Some(i) = insert {
                if self.match_byte(pos, EMPTY) != 0 {
                    return Err(self.fix_insert_index(i));
                }
            }
            stride += WIDTH;
            pos = (pos + stride) & self.bucket_mask;
        }
    }

    /// Stores `value` in the insert bucket `entry` returned.
    pub(crate) fn insert_at(&mut self, hash: u64, bucket: usize, value: usize) {
        if self.ctrl[bucket] == EMPTY {
            self.growth_left -= 1;
        }
        self.set_ctrl(bucket, tag(hash));
        self.slots[bucket] = value;
        self.items += 1;
    }

    /// HashTable::insert_unique: a value whose key is known to be absent.
    pub(crate) fn insert_unique<H: Fn(usize) -> u64>(&mut self, hash: u64, value: usize, hasher: H) -> usize {
        let mut index = self.find_insert_index(hash);
        if self.growth_left == 0 && self.ctrl[index] == EMPTY {
            self.reserve(1, hasher);
            index = self.find_insert_index(hash);
        }
        self.insert_at(hash, index, value);
        index
    }

    /// Removes the item in a full bucket: EMPTY when no probe sequence
    /// can pass this bucket, else a DELETED tombstone.
    pub(crate) fn erase(&mut self, index: usize) {
        let before = index.wrapping_sub(WIDTH) & self.bucket_mask;
        let empty_before = self.match_byte(before, EMPTY);
        let empty_after = self.match_byte(index, EMPTY);
        let leading = if empty_before == 0 { WIDTH } else { WIDTH - 1 - (31 - empty_before.leading_zeros() as usize) };
        let trailing = if empty_after == 0 { WIDTH } else { empty_after.trailing_zeros() as usize };
        let c = if leading + trailing >= WIDTH {
            DELETED
        } else {
            self.growth_left += 1;
            EMPTY
        };
        self.set_ctrl(index, c);
        self.items -= 1;
    }

    /// HashTable::clear: no-op on an empty table (tombstones stay).
    pub(crate) fn clear(&mut self) {
        if self.items == 0 {
            return;
        }
        if !self.is_singleton() {
            let mut i = 0;
            while i < self.ctrl.len() {
                self.ctrl[i] = EMPTY;
                i += 1;
            }
        }
        self.items = 0;
        self.growth_left = bucket_mask_to_capacity(self.bucket_mask);
    }

    /// Room for `additional` more items without growing.
    pub(crate) fn reserve<H: Fn(usize) -> u64>(&mut self, additional: usize, hasher: H) {
        if additional > self.growth_left {
            let new_items = match self.items.checked_add(additional) {
                Some(n) => n,
                None => panic!("Hash table capacity overflow"),
            };
            let full_capacity = bucket_mask_to_capacity(self.bucket_mask);
            if new_items <= full_capacity / 2 {
                self.rehash_in_place(hasher);
            } else {
                let cap = if new_items > full_capacity + 1 { new_items } else { full_capacity + 1 };
                self.resize(cap, hasher);
            }
        }
    }

    /// HashTable::shrink_to.
    pub(crate) fn shrink_to<H: Fn(usize) -> u64>(&mut self, min_capacity: usize, hasher: H) {
        let min_size = if self.items > min_capacity { self.items } else { min_capacity };
        if min_size == 0 {
            *self = Indices::new();
            return;
        }
        let min_buckets = match capacity_to_buckets(min_size) {
            Some(b) => b,
            None => return,
        };
        if min_buckets < self.num_buckets() {
            if self.items == 0 {
                *self = Indices::with_capacity(min_size);
            } else {
                self.resize(min_size, hasher);
            }
        }
    }

    /// Moves every item, in bucket order, into a new table.
    fn resize<H: Fn(usize) -> u64>(&mut self, capacity: usize, hasher: H) {
        let mut nt = Indices::with_capacity(capacity);
        // The control bytes and the slots side by side: no check per bucket
        // (the singleton has no slots, so the zip is empty there).
        for (&c, &v) in self.ctrl.iter().zip(self.slots.iter()) {
            if is_full(c) {
                let h = hasher(v);
                let ni = nt.find_insert_index(h);
                nt.set_ctrl(ni, tag(h));
                nt.slots[ni] = v;
            }
        }
        nt.growth_left -= self.items;
        nt.items = self.items;
        *self = nt;
    }

    /// hashbrown's rehash_in_place: drops tombstones without growing.
    fn rehash_in_place<H: Fn(usize) -> u64>(&mut self, hasher: H) {
        let buckets = self.num_buckets();
        let mut i = 0;
        while i < buckets {
            self.ctrl[i] = if is_full(self.ctrl[i]) { DELETED } else { EMPTY };
            i += 1;
        }
        if buckets < WIDTH {
            i = 0;
            while i < buckets {
                self.ctrl[WIDTH + i] = self.ctrl[i];
                i += 1;
            }
        } else {
            i = 0;
            while i < WIDTH {
                self.ctrl[buckets + i] = self.ctrl[i];
                i += 1;
            }
        }
        i = 0;
        while i < buckets {
            if self.ctrl[i] == DELETED {
                loop {
                    let h = hasher(self.slots[i]);
                    let new_i = self.find_insert_index(h);
                    let pos0 = (h as usize) & self.bucket_mask;
                    let same_group = (i.wrapping_sub(pos0) & self.bucket_mask) / WIDTH == (new_i.wrapping_sub(pos0) & self.bucket_mask) / WIDTH;
                    if same_group {
                        self.set_ctrl(i, tag(h));
                        break;
                    }
                    let prev = self.ctrl[new_i];
                    self.set_ctrl(new_i, tag(h));
                    if prev == EMPTY {
                        self.set_ctrl(i, EMPTY);
                        self.slots[new_i] = self.slots[i];
                        break;
                    }
                    let t = self.slots[i];
                    self.slots[i] = self.slots[new_i];
                    self.slots[new_i] = t;
                }
            }
            i += 1;
        }
        self.growth_left = bucket_mask_to_capacity(self.bucket_mask) - self.items;
    }

    /// Visits every full bucket in bucket order (hashbrown's iteration
    /// order); `f` may change the entry index stored there.
    pub(crate) fn for_each_mut<F: FnMut(&mut usize)>(&mut self, mut f: F) {
        for (&c, slot) in self.ctrl.iter().zip(self.slots.iter_mut()) {
            if is_full(c) {
                f(slot);
            }
        }
    }

    /// HashTable::retain: erases, in bucket order, every item `keep` rejects.
    pub(crate) fn retain<F: FnMut(&mut usize) -> bool>(&mut self, mut keep: F) {
        if self.is_singleton() {
            return;
        }
        let mut i = 0;
        while i < self.num_buckets() {
            if is_full(self.ctrl[i]) && !keep(&mut self.slots[i]) {
                self.erase(i);
            }
            i += 1;
        }
    }
}
