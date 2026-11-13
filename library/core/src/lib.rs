// Lust `core`: no OS, no alloc (docs/ARCH.md section 3.1).
//
// Skeleton from serial refactor S5. Traits, Option, Result, iterators,
// fmt, and the slice/str/mem/cmp/hash/num surface land here with W-STD,
// after the minimal generic pipeline passes (section 9.3). Items the
// compiler must know by def are marked `#[lust_lang = "..."]` on the
// line above the item; the loader reads the marks at std load
// (src/stdload.rs). The mark list is `LANG_ITEMS` in src/ty/lang.rs.
