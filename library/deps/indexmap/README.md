# Adapter: indexmap 2.14.0

Lust adapter crate (docs/ARCH.md section 3.6). Gate:
`scripts/test-adapters.sh indexmap`.

## Implemented, same results as upstream

- `IndexMap<K, V, S = RandomState>` (the default hasher, `new`, and
  `with_capacity` need feature `std`, as upstream): `new`, `with_capacity`,
  `with_hasher`, `with_capacity_and_hasher`, `capacity`, `hasher`,
  `len`, `is_empty`, `iter`, `keys`, `values`, `clear`, `truncate`,
  `split_off`, `reserve`, `reserve_exact`, `shrink_to_fit`, `shrink_to`,
  `insert`, `insert_full`, `entry` (`or_insert`, `or_insert_with`,
  `or_default`, `and_modify`, `key`, `index`, occupied `get`,
  `get_mut`, `into_mut`, `insert`, `swap_remove`, `shift_remove`,
  vacant `insert`), `contains_key`, `get`, `get_key_value`, `get_full`,
  `get_index_of`, `get_mut`, `iter_mut`, `values_mut`, `into_keys`,
  `into_values`, `remove` (deprecated, swap), `swap_remove`,
  `swap_remove_entry`, `swap_remove_full`, `shift_remove`,
  `shift_remove_entry`, `shift_remove_full`, `pop`, `retain`,
  `sort_keys`, `sort_by`, `sort_by_key`, `sort_unstable_keys`,
  `sort_unstable_by`, `sort_unstable_by_key`, `reverse`, `get_index`,
  `get_index_mut`, `first`, `last`, `first_mut`, `last_mut`,
  `swap_remove_index`, `shift_remove_index`, `move_index`,
  `swap_indices`, `as_slice`; `IntoIterator` (by value, `&`, `&mut`), `Extend`, `FromIterator`, `Index` (by key, and by position with `IndexMut`),
  `Clone`, `Default`, `Debug`, `PartialEq`, `Eq`.
- `IndexSet<T, S = RandomState>` over `IndexMap<T, (), S>`, as upstream.
- `map::Slice<K, V>`, an unsized newtype over the entries as upstream:
  `len`, `is_empty`, `first`, `last` (const, as upstream), `get_index`,
  `iter`, `keys`, `values`, `IntoIterator for &Slice`, `Debug`,
  `PartialEq<Slice<K2, V2>>` (order counts, upstream's `slice_eq`), `Eq`.
- The hash index is hashbrown 0.17.1's table logic for usize slots and
  the core is upstream's src/inner.rs, so order, capacity, and the Hash
  and Eq calls a key sees match upstream for the same hasher. Checked
  with rustc against the real crate: 133,010 index steps vs hashbrown,
  93,446 core steps, 23,279 map and 23,575 set API steps with exact
  Hash/Eq call counts.

## Rejected (lust error, never a different answer)

- `Equivalent` (lookups take `K: Borrow<Q>`, as std maps do), the other
  `Slice` items (mutable and boxed slices, ranges, ordering, hashing),
  `drain`, `splice`, `extract_if`, raw entries, sorted inserts,
  `insert_before`, `shift_insert`, `replace_index`, `get_disjoint_mut`,
  the `indexmap!`/`indexset!` macros, and the rayon, serde, borsh,
  arbitrary, quickcheck, and sval features. Layout queries reject.
