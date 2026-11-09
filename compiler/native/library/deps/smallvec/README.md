# Adapter: smallvec 1.15.1

Lust adapter crate (docs/ARCH.md section 3.6) with upstream's public
shape: `SmallVec<A>` with `A: Array` (`SmallVec<[TypeId; 16]>`), the
`Array` impls for upstream's sizes (0 to 32, 36, 64, 96, 128, then
powers of two to 2^20, as without `const_generics`), and `smallvec!`.
Gate: `scripts/test-adapters.sh smallvec`.

## Implemented, same results as upstream

- `new`, `with_capacity`, `from_vec`, `from_slice`, `from_elem`,
  `inline_size`, `len`, `is_empty`, `capacity`, `spilled`, `push`,
  `pop`, `grow`, `reserve`, `reserve_exact`, `shrink_to_fit`,
  `truncate`, `clear`, `swap_remove`, `remove`, `insert`,
  `insert_from_slice`, `extend_from_slice`, `resize`, `retain`,
  `retain_mut`, `dedup`, `dedup_by`, `dedup_by_key`, `drain` (and its
  `Drain` iterator), `into_vec`, `as_slice`, `as_mut_slice`;
  `Deref`/`DerefMut` to the slice; `Extend`,
  `FromIterator`, `IntoIterator` (by value, `&`, `&mut`), `Clone`,
  `Default`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`,
  `From<Vec>`, `From<&[T]>`.
- Capacity is upstream's logical capacity (inline size until the spill,
  then power-of-two growth on push and reserve, exact on reserve_exact
  and shrink_to_fit, back inline when it fits). Drop and clone order
  are upstream's (truncate, clear, retain, and dedup drop from the
  back; resize clones then drops its value). Checked with rustc against
  the real crate over 59,421 random steps for N = 0, 1, 4, 8, 16.
- Items live in a Vec (no inline buffer), so layout queries on
  SmallVec reject (L0652).

## Rejected (lust error, never a different answer)

- `drain_filter`, `insert_many`, `append`, `into_inner`,
  `into_boxed_slice`, `from_buf`, `from_buf_and_len`, raw pointers and
  parts, `set_len`, the const forms, and the `union`, `const_generics`,
  `write`, `serde`, and `arbitrary` features.
