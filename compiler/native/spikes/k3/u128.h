// K3: two-word 128-bit helpers for tcc (no __int128). Layout matches Rust
// u128/i128 on little-endian x86_64: 16 bytes, low word first. The struct has
// only 8-byte natural alignment in C; codegen must add explicit 16-byte
// alignment where the Rust layout needs it (see align.c).
#ifndef LUST_U128_H
#define LUST_U128_H
#include <stdint.h>
#include <string.h>
typedef struct { uint64_t lo, hi; } lust_u128;
typedef lust_u128 lust_i128;
static inline lust_u128 u128_make(uint64_t lo, uint64_t hi) { lust_u128 r; r.lo = lo; r.hi = hi; return r; }
static inline lust_u128 u128_add(lust_u128 a, lust_u128 b) {
    lust_u128 r; r.lo = a.lo + b.lo; r.hi = a.hi + b.hi + (r.lo < a.lo); return r;
}
static inline lust_u128 u128_sub(lust_u128 a, lust_u128 b) {
    lust_u128 r; r.lo = a.lo - b.lo; r.hi = a.hi - b.hi - (a.lo < b.lo); return r;
}
static inline lust_u128 u128_neg(lust_u128 a) { return u128_sub(u128_make(0, 0), a); }
static inline lust_u128 u128_mul(lust_u128 a, lust_u128 b) {
    // 32-bit limbs: products of limbs fit in uint64_t.
    uint64_t a0 = (uint32_t)a.lo, a1 = a.lo >> 32, a2 = (uint32_t)a.hi, a3 = a.hi >> 32;
    uint64_t b0 = (uint32_t)b.lo, b1 = b.lo >> 32, b2 = (uint32_t)b.hi, b3 = b.hi >> 32;
    uint64_t p00 = a0 * b0, p01 = a0 * b1, p10 = a1 * b0, p11 = a1 * b1;
    uint64_t mid = (p00 >> 32) + (uint32_t)p01 + (uint32_t)p10;
    uint64_t lo = (p00 & 0xffffffffull) | (mid << 32);
    // hi covers limb weights 2 and 3; weight >= 4 terms drop mod 2^128.
    uint64_t hi = p11 + (p01 >> 32) + (p10 >> 32) + (mid >> 32) + a0 * b2 + a2 * b0
         + ((a0 * b3 + a3 * b0 + a1 * b2 + a2 * b1) << 32);
    return u128_make(lo, hi);
}
static inline int u128_cmp(lust_u128 a, lust_u128 b) {
    if (a.hi != b.hi) return a.hi < b.hi ? -1 : 1;
    if (a.lo != b.lo) return a.lo < b.lo ? -1 : 1;
    return 0;
}
static inline int i128_cmp(lust_i128 a, lust_i128 b) {
    int64_t ah = (int64_t)a.hi, bh = (int64_t)b.hi;
    if (ah != bh) return ah < bh ? -1 : 1;
    if (a.lo != b.lo) return a.lo < b.lo ? -1 : 1;
    return 0;
}
static inline lust_u128 u128_shl(lust_u128 a, unsigned n) {
    n &= 127;
    if (n == 0) return a;
    if (n >= 64) return u128_make(0, a.lo << (n - 64));
    return u128_make(a.lo << n, (a.hi << n) | (a.lo >> (64 - n)));
}
static inline lust_u128 u128_shr(lust_u128 a, unsigned n) {
    n &= 127;
    if (n == 0) return a;
    if (n >= 64) return u128_make(a.hi >> (n - 64), 0);
    return u128_make((a.lo >> n) | (a.hi << (64 - n)), a.hi >> n);
}
static inline lust_i128 i128_sar(lust_i128 a, unsigned n) {
    n &= 127;
    if (n == 0) return a;
    int neg = (int64_t)a.hi < 0;
    if (n >= 64) return u128_make((uint64_t)((int64_t)a.hi >> (n - 64)), neg ? ~0ull : 0);
    return u128_make((a.lo >> n) | (a.hi << (64 - n)), (uint64_t)((int64_t)a.hi >> n));
}
// Restoring long division. Caller guarantees b != 0 (Rust panics first).
static inline lust_u128 u128_divmod(lust_u128 a, lust_u128 b, lust_u128 *rem) {
    lust_u128 q = u128_make(0, 0), r = u128_make(0, 0);
    for (int i = 127; i >= 0; i--) {
        r = u128_shl(r, 1);
        uint64_t bit = i >= 64 ? (a.hi >> (i - 64)) & 1ull : (a.lo >> i) & 1ull;
        r.lo |= bit;
        if (u128_cmp(r, b) >= 0) {
            r = u128_sub(r, b);
            if (i >= 64) q.hi |= 1ull << (i - 64); else q.lo |= 1ull << i;
        }
    }
    if (rem) *rem = r;
    return q;
}
static inline lust_u128 u128_div(lust_u128 a, lust_u128 b) { return u128_divmod(a, b, 0); }
static inline lust_u128 u128_rem(lust_u128 a, lust_u128 b) { lust_u128 r; (void)u128_divmod(a, b, &r); return r; }
static inline lust_i128 i128_absu(lust_i128 a) { return (int64_t)a.hi < 0 ? u128_neg(a) : a; }
static inline lust_i128 i128_div(lust_i128 a, lust_i128 b) {
    lust_u128 q = u128_divmod(i128_absu(a), i128_absu(b), 0);
    return ((int64_t)a.hi < 0) != ((int64_t)b.hi < 0) ? u128_neg(q) : q;
}
static inline lust_i128 i128_rem(lust_i128 a, lust_i128 b) {
    lust_u128 r; (void)u128_divmod(i128_absu(a), i128_absu(b), &r);
    return (int64_t)a.hi < 0 ? u128_neg(r) : r;
}
// Format as decimal into out (40 bytes is enough: 39 digits + NUL; 41 with sign).
static inline void u128_format(lust_u128 a, char *out) {
    char tmp[40]; int n = 0;
    if (a.lo == 0 && a.hi == 0) { out[0] = '0'; out[1] = 0; return; }
    lust_u128 ten = u128_make(10, 0);
    while (a.lo || a.hi) { lust_u128 r; a = u128_divmod(a, ten, &r); tmp[n++] = (char)('0' + r.lo); }
    for (int i = 0; i < n; i++) out[i] = tmp[n - 1 - i];
    out[n] = 0;
}
static inline void i128_format(lust_i128 a, char *out) {
    if ((int64_t)a.hi < 0) { out[0] = '-'; u128_format(u128_neg(a), out + 1); }
    else u128_format(a, out);
}
#endif
