// Lust `alloc`: heap types (docs/ARCH.md section 3.1).
//
// Skeleton from serial refactor S5. String, Vec, Box, Rc, Arc, the
// collections, format!, Cow, and BTreeMap land here with W-STD, over
// the erased C cores of section 3.3. Until then String and Vec stay on
// their builtin paths in check, lower, and cgen; no Builtin case is
// deleted by S5.
