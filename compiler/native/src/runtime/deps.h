/* Adapter crate cores (docs/ARCH.md section 3.6, package W-DEPS): the C
   behind the adapter intrinsics of src/ty/lang.rs. Each one is exact
   with the upstream crate version its adapter pins. A `&[u8]` intrinsic
   argument (SigTy::Bytes) arrives here as a pointer and a length.

   Declarations here are in every unit; bodies (deps.c) only in the
   runtime unit (LUST_IMPL), like fmt.h. */
#ifndef LUST_RT_DEPS_H
#define LUST_RT_DEPS_H

/* memchr 2.8.2 adapter: the index of the first (memchr) or last
   (memrchr) byte equal to `needle`, or UINT64_MAX for none; memmem: the
   index of the first occurrence of the needle (0 for an empty needle),
   or UINT64_MAX. */
uint64_t lust_memchr(uint32_t needle, const uint8_t *p, uint64_t len);
uint64_t lust_memrchr(uint32_t needle, const uint8_t *p, uint64_t len);
uint64_t lust_memmem(const uint8_t *h, uint64_t hlen, const uint8_t *n, uint64_t nlen);
/* The first or last index of a byte equal to one of three needles, or UINT64_MAX. */
uint64_t lust_memchr3(uint32_t a, uint32_t b, uint32_t c, const uint8_t *p, uint64_t n);
uint64_t lust_memrchr3(uint32_t a, uint32_t b, uint32_t c, const uint8_t *p, uint64_t n);

/* XXH3 64-bit and 128-bit of `len` bytes, exact with xxhash-rust 0.8.15:
   xxh3_64 and xxh3_128 for seed 0, the _with_seed forms otherwise. */
uint64_t lust_xxh3_64(const uint8_t *p, uint64_t len, uint64_t seed);
lust_u128 lust_xxh3_128(const uint8_t *p, uint64_t len, uint64_t seed);

#ifdef LUST_IMPL
#include "deps.c"
#endif

#endif
