/* Bodies of the adapter crate cores declared in deps.h (W-DEPS). */
#include <string.h>

/* memchr through libc. A zero length never touches the pointer (an
   empty Rust slice may hold a dangling one). */
uint64_t lust_memchr(uint32_t needle, const uint8_t *p, uint64_t len) {
    const uint8_t *r = len ? (const uint8_t *)memchr(p, (int)(needle & 0xFFu), (size_t)len) : NULL;
    return r ? (uint64_t)(r - p) : UINT64_MAX;
}

/* memrchr is a GNU extension; a backward loop is portable. */
uint64_t lust_memrchr(uint32_t needle, const uint8_t *p, uint64_t len) {
    uint8_t b = (uint8_t)needle;
    while (len) {
        len--;
        if (p[len] == b) return len;
    }
    return UINT64_MAX;
}

#if !defined(__TINYC__) && defined(__x86_64__) && defined(__SSE2__)
/* Byte ranks for memmem's prefilter, 0 the rarest: the byte frequencies of TypeScript
   declarations (tsgo's bundled lib files) and Rust source, the text tsgo searches. */
static const uint8_t lust_mm_rank[256] = {
      0,   1,   2,   3,   4,   5,   6,   7,   8, 158, 245,   9,  10,  11,  12,  13,
     14,  15,  16,  17,  18,  19,  20,  21,  22,  23,  24,  25,  26,  27,  28,  29,
    255, 185, 208, 174, 161, 159, 204, 182, 232, 233, 226, 164, 230, 202, 234, 236,
    215, 197, 192, 183, 181, 176, 180, 172, 177, 173, 237, 225, 193, 219, 203, 175,
    163, 212, 187, 209, 206, 220, 198, 199, 179, 210, 171, 190, 201, 200, 217, 207,
    205, 162, 211, 223, 222, 184, 188, 189, 167, 165, 160, 195, 166, 194, 156, 242,
    216, 248, 228, 243, 244, 254, 240, 231, 235, 247, 186, 218, 246, 238, 251, 252,
    241, 178, 250, 249, 253, 239, 224, 221, 227, 229, 191, 214, 196, 213, 155,  30,
    168,  31, 157,  97, 112, 135, 153, 113, 144, 114,  32, 136, 146, 125,  33,  34,
    145,  35, 154, 138, 169,  98, 142, 134, 152, 115,  36,  37, 143, 116, 117, 139,
     38,  99,  39, 118,  40, 126, 147,  41, 100, 149,  42, 101, 127,  43,  44,  45,
    150, 120, 121, 102,  46,  47,  48,  49, 103, 104, 137, 128, 122, 129,  50, 130,
     51,  52, 141, 151, 148, 105,  53,  54,  55,  56,  57,  58, 131,  59,  60, 119,
     61,  62,  63,  64,  65,  66,  67,  68,  69,  70,  71,  72,  73,  74,  75,  76,
     77, 132, 170, 106, 107, 133, 108, 123, 109, 110,  78,  79,  80, 111,  81, 124,
    140,  82,  83,  84,  85,  86,  87,  88,  89,  90,  91,  92,  93,  94,  95,  96,
};
typedef char lust_mm_v16 __attribute__((vector_size(16)));
typedef char lust_mm_v16u __attribute__((vector_size(16), aligned(1)));
/* lust_memmem's block: starts i..i+15, the pair bytes compared at their offsets, then the
   needle at each start where both match (in order); the first match or UINT64_MAX. A
   2-byte needle's pair is the whole needle: a pair match is a match. */
static inline uint64_t lust_mm_block(const uint8_t *h, uint64_t i, uint64_t i1, uint64_t i2, lust_mm_v16 b1,
                                     lust_mm_v16 b2, const uint8_t *n, uint64_t nlen) {
    lust_mm_v16 e1 = *(const lust_mm_v16u *)(const void *)(h + i + i1) == b1;
    lust_mm_v16 e2 = *(const lust_mm_v16u *)(const void *)(h + i + i2) == b2;
    for (unsigned m = (unsigned)__builtin_ia32_pmovmskb128(e1 & e2); m; m &= m - 1) {
        uint64_t j = i + (uint64_t)__builtin_ctz(m);
        if (nlen == 2 || memcmp(h + j, n, (size_t)nlen) == 0) return j;
    }
    return UINT64_MAX;
}
#endif

/* memmem is a GNU extension too. The first match is the answer: candidates are tried in
   order. A clang or gcc build for x86_64 prefilters 16 candidate starts at a time on two
   bytes of the needle (the memchr crate's packed-pair idea: for a needle longer than 2,
   the bytes with the lowest `lust_mm_rank`, at their offsets), so the compare runs only
   where both match; the last block ends at the last start (it can overlap the block
   before: a start tried twice fails twice). It never reads past the haystack. Other builds
   take libc memchr for the first byte and memcmp to confirm. */
uint64_t lust_memmem(const uint8_t *h, uint64_t hlen, const uint8_t *n, uint64_t nlen) {
    if (nlen == 0) return 0;
    if (nlen > hlen) return UINT64_MAX;
    uint64_t last = hlen - nlen, i = 0;
#if !defined(__TINYC__) && defined(__x86_64__) && defined(__SSE2__)
    if (nlen >= 2 && last >= 16) {
        /* i1: the rarest byte; i2: the rarest other byte value (any other offset when
           the needle repeats one byte). */
        uint64_t i1 = 0, i2 = 1;
        if (nlen > 2) {
            for (uint64_t k = 1; k < nlen; k++)
                if (lust_mm_rank[n[k]] < lust_mm_rank[n[i1]]) i1 = k;
            i2 = i1 == 0 ? 1 : 0;
            int other = 0;
            for (uint64_t k = 0; k < nlen; k++)
                if (n[k] != n[i1] && (!other || lust_mm_rank[n[k]] < lust_mm_rank[n[i2]])) i2 = k, other = 1;
        }
        lust_mm_v16 b1 = (lust_mm_v16){0} + (char)n[i1], b2 = (lust_mm_v16){0} + (char)n[i2];
        /* Blocks at i < end, then the last one at end. The loads end at
           h + end + 15 + max(i1, i2) <= h + last + nlen - 1 = h + hlen - 1. */
        uint64_t end = last - 15, r;
        for (; i < end; i += 16)
            if ((r = lust_mm_block(h, i, i1, i2, b1, b2, n, nlen)) != UINT64_MAX) return r;
        return lust_mm_block(h, end, i1, i2, b1, b2, n, nlen);
    }
#endif
    while (i <= last) {
        const uint8_t *f = (const uint8_t *)memchr(h + i, n[0], (size_t)(last - i + 1));
        if (!f) return UINT64_MAX;
        i = (uint64_t)(f - h);
        if (memcmp(h + i, n, (size_t)nlen) == 0) return i;
        i++;
    }
    return UINT64_MAX;
}

/* The first (memchr3) or last (memrchr3) index of a byte equal to one of three needles,
   or UINT64_MAX: the memchr port's 2- and 3-needle searches and their iterators. A clang
   or gcc build for x86_64 tests 16 bytes a step (three compares, one mask), and the bytes
   left are one more block that ends at the slice's end (memrchr3: starts at its start),
   which may test bytes again: they hold no match, so the answer is the same. Other builds
   loop over the bytes. It never reads outside the n bytes. */
uint64_t lust_memchr3(uint32_t a, uint32_t b, uint32_t c, const uint8_t *p, uint64_t n) {
    uint64_t i = 0;
#if !defined(__TINYC__) && defined(__x86_64__) && defined(__SSE2__)
    if (n >= 16) {
        lust_mm_v16 va = (lust_mm_v16){0} + (char)a, vb = (lust_mm_v16){0} + (char)b, vc = (lust_mm_v16){0} + (char)c;
        for (; i + 16 <= n; i += 16) {
            lust_mm_v16 x = *(const lust_mm_v16u *)(const void *)(p + i);
            unsigned m = (unsigned)__builtin_ia32_pmovmskb128((x == va) | (x == vb) | (x == vc));
            if (m) return i + (uint64_t)__builtin_ctz(m);
        }
        if (i == n) return UINT64_MAX;
        lust_mm_v16 x = *(const lust_mm_v16u *)(const void *)(p + n - 16);
        unsigned m = (unsigned)__builtin_ia32_pmovmskb128((x == va) | (x == vb) | (x == vc));
        return m ? n - 16 + (uint64_t)__builtin_ctz(m) : UINT64_MAX;
    }
#endif
    for (; i < n; i++)
        if (p[i] == (uint8_t)a || p[i] == (uint8_t)b || p[i] == (uint8_t)c) return i;
    return UINT64_MAX;
}

uint64_t lust_memrchr3(uint32_t a, uint32_t b, uint32_t c, const uint8_t *p, uint64_t n) {
#if !defined(__TINYC__) && defined(__x86_64__) && defined(__SSE2__)
    if (n >= 16) {
        lust_mm_v16 va = (lust_mm_v16){0} + (char)a, vb = (lust_mm_v16){0} + (char)b, vc = (lust_mm_v16){0} + (char)c;
        uint64_t e = n;
        for (; e >= 16; e -= 16) {
            lust_mm_v16 x = *(const lust_mm_v16u *)(const void *)(p + e - 16);
            unsigned m = (unsigned)__builtin_ia32_pmovmskb128((x == va) | (x == vb) | (x == vc));
            if (m) return e - 16 + (uint64_t)(31 - __builtin_clz(m));
        }
        if (e == 0) return UINT64_MAX;
        lust_mm_v16 x = *(const lust_mm_v16u *)(const void *)p;
        unsigned m = (unsigned)__builtin_ia32_pmovmskb128((x == va) | (x == vb) | (x == vc));
        return m ? (uint64_t)(31 - __builtin_clz(m)) : UINT64_MAX;
    }
#endif
    while (n) {
        n--;
        if (p[n] == (uint8_t)a || p[n] == (uint8_t)b || p[n] == (uint8_t)c) return n;
    }
    return UINT64_MAX;
}

/* XXH3 one-shot hashes for the xxhash-rust 0.8.15 adapter. A scalar
   port of xxhash-rust's src/xxh3.rs (the default secret, seeded and
   unseeded forms); its SIMD paths compute the same values. No
   __int128: tcc lacks it, so the 64x64->128 product is built from
   32-bit halves. Inputs are read as little-endian bytes, so the result
   does not depend on the host. Gate: scripts/test-adapters.sh
   xxhash-rust (the real crate is the oracle). */

#define XXH_P32_1 0x9E3779B1u
#define XXH_P32_2 0x85EBCA77u
#define XXH_P32_3 0xC2B2AE3Du
#define XXH_P64_1 0x9E3779B185EBCA87ull
#define XXH_P64_2 0xC2B2AE3D27D4EB4Full
#define XXH_P64_3 0x165667B19E3779F9ull
#define XXH_P64_4 0x85EBCA77C2B2AE63ull
#define XXH_P64_5 0x27D4EB2F165667C5ull
#define XXH_SECRET_SIZE 192
#define XXH_SECRET_SIZE_MIN 136
#define XXH_STRIPE_LEN 64
#define XXH_CONSUME_RATE 8
#define XXH_MERGEACCS_START 11
#define XXH_LASTACC_START 7

static const uint8_t xxh_default_secret[XXH_SECRET_SIZE] = {
    0xb8, 0xfe, 0x6c, 0x39, 0x23, 0xa4, 0x4b, 0xbe, 0x7c, 0x01, 0x81, 0x2c, 0xf7, 0x21, 0xad, 0x1c,
    0xde, 0xd4, 0x6d, 0xe9, 0x83, 0x90, 0x97, 0xdb, 0x72, 0x40, 0xa4, 0xa4, 0xb7, 0xb3, 0x67, 0x1f,
    0xcb, 0x79, 0xe6, 0x4e, 0xcc, 0xc0, 0xe5, 0x78, 0x82, 0x5a, 0xd0, 0x7d, 0xcc, 0xff, 0x72, 0x21,
    0xb8, 0x08, 0x46, 0x74, 0xf7, 0x43, 0x24, 0x8e, 0xe0, 0x35, 0x90, 0xe6, 0x81, 0x3a, 0x26, 0x4c,
    0x3c, 0x28, 0x52, 0xbb, 0x91, 0xc3, 0x00, 0xcb, 0x88, 0xd0, 0x65, 0x8b, 0x1b, 0x53, 0x2e, 0xa3,
    0x71, 0x64, 0x48, 0x97, 0xa2, 0x0d, 0xf9, 0x4e, 0x38, 0x19, 0xef, 0x46, 0xa9, 0xde, 0xac, 0xd8,
    0xa8, 0xfa, 0x76, 0x3f, 0xe3, 0x9c, 0x34, 0x3f, 0xf9, 0xdc, 0xbb, 0xc7, 0xc7, 0x0b, 0x4f, 0x1d,
    0x8a, 0x51, 0xe0, 0x4b, 0xcd, 0xb4, 0x59, 0x31, 0xc8, 0x9f, 0x7e, 0xc9, 0xd9, 0x78, 0x73, 0x64,
    0xea, 0xc5, 0xac, 0x83, 0x34, 0xd3, 0xeb, 0xc3, 0xc5, 0x81, 0xa0, 0xff, 0xfa, 0x13, 0x63, 0xeb,
    0x17, 0x0d, 0xdd, 0x51, 0xb7, 0xf0, 0xda, 0x49, 0xd3, 0x16, 0x55, 0x26, 0x29, 0xd4, 0x68, 0x9e,
    0x2b, 0x16, 0xbe, 0x58, 0x7d, 0x47, 0xa1, 0xfc, 0x8f, 0xf8, 0xb8, 0xd1, 0x7a, 0xd0, 0x31, 0xce,
    0x45, 0xcb, 0x3a, 0x8f, 0x95, 0x16, 0x04, 0x28, 0xaf, 0xd7, 0xfb, 0xca, 0xbb, 0x4b, 0x40, 0x7e,
};

static uint32_t xxh_r32(const uint8_t *p) {
    return (uint32_t)p[0] | (uint32_t)p[1] << 8 | (uint32_t)p[2] << 16 | (uint32_t)p[3] << 24;
}

static uint64_t xxh_r64(const uint8_t *p) {
    return (uint64_t)xxh_r32(p) | (uint64_t)xxh_r32(p + 4) << 32;
}

static uint32_t xxh_swap32(uint32_t x) {
    return (x >> 24) | ((x >> 8) & 0xff00u) | ((x << 8) & 0xff0000u) | (x << 24);
}

static uint64_t xxh_swap64(uint64_t x) {
    return (uint64_t)xxh_swap32((uint32_t)x) << 32 | xxh_swap32((uint32_t)(x >> 32));
}

static uint64_t xxh_rotl64(uint64_t x, int r) { return (x << r) | (x >> (64 - r)); }
static uint32_t xxh_rotl32(uint32_t x, int r) { return (x << r) | (x >> (32 - r)); }

/* The full 128-bit product of two u64 values. */
static void xxh_mul128(uint64_t a, uint64_t b, uint64_t *lo, uint64_t *hi) {
    uint64_t a_lo = (uint32_t)a, a_hi = a >> 32, b_lo = (uint32_t)b, b_hi = b >> 32;
    uint64_t lo_lo = a_lo * b_lo, hi_lo = a_hi * b_lo, lo_hi = a_lo * b_hi, hi_hi = a_hi * b_hi;
    uint64_t cross = (lo_lo >> 32) + (uint32_t)hi_lo + lo_hi;
    *hi = (hi_lo >> 32) + (cross >> 32) + hi_hi;
    *lo = (cross << 32) | (uint32_t)lo_lo;
}

static uint64_t xxh_fold64(uint64_t a, uint64_t b) {
    uint64_t lo, hi;
    xxh_mul128(a, b, &lo, &hi);
    return lo ^ hi;
}

static uint64_t xxh_xorshift(uint64_t v, int s) { return v ^ (v >> s); }

static uint64_t xxh3_avalanche(uint64_t v) {
    v = xxh_xorshift(v, 37) * 0x165667919E3779F9ull;
    return xxh_xorshift(v, 32);
}

static uint64_t xxh64_avalanche(uint64_t v) {
    v ^= v >> 33;
    v *= XXH_P64_2;
    v ^= v >> 29;
    v *= XXH_P64_3;
    return v ^ (v >> 32);
}

static uint64_t xxh_strong_avalanche(uint64_t v, uint64_t len) {
    v ^= xxh_rotl64(v, 49) ^ xxh_rotl64(v, 24);
    v *= 0x9FB21C651E98DF25ull;
    v ^= (v >> 35) + len;
    v *= 0x9FB21C651E98DF25ull;
    return xxh_xorshift(v, 28);
}

static uint64_t xxh_mix16(const uint8_t *in, const uint8_t *sec, uint64_t seed) {
    uint64_t lo = xxh_r64(in) ^ (xxh_r64(sec) + seed);
    uint64_t hi = xxh_r64(in + 8) ^ (xxh_r64(sec + 8) - seed);
    return xxh_fold64(lo, hi);
}

static void xxh_mix32(uint64_t *lo, uint64_t *hi, const uint8_t *in1, const uint8_t *in2, const uint8_t *sec, uint64_t seed) {
    *lo += xxh_mix16(in1, sec, seed);
    *lo ^= xxh_r64(in2) + xxh_r64(in2 + 8);
    *hi += xxh_mix16(in2, sec + 16, seed);
    *hi ^= xxh_r64(in1) + xxh_r64(in1 + 8);
}

/* The default secret with `seed` added to the low and subtracted from
   the high u64 of each 16-byte pair (custom_default_secret). */
static void xxh_seeded_secret(uint64_t seed, uint8_t *out) {
    for (int i = 0; i < XXH_SECRET_SIZE; i += 16) {
        uint64_t lo = xxh_r64(xxh_default_secret + i) + seed, hi = xxh_r64(xxh_default_secret + i + 8) - seed;
        for (int b = 0; b < 8; b++) {
            out[i + b] = (uint8_t)(lo >> (8 * b));
            out[i + 8 + b] = (uint8_t)(hi >> (8 * b));
        }
    }
}

static void xxh_accumulate512(uint64_t *acc, const uint8_t *in, const uint8_t *sec) {
    for (int i = 0; i < 8; i++) {
        uint64_t v = xxh_r64(in + 8 * i), k = v ^ xxh_r64(sec + 8 * i);
        acc[i ^ 1] += v;
        acc[i] += (uint64_t)(uint32_t)k * (k >> 32);
    }
}

static void xxh_scramble(uint64_t *acc, const uint8_t *sec) {
    for (int i = 0; i < 8; i++) {
        acc[i] = (xxh_xorshift(acc[i], 47) ^ xxh_r64(sec + 8 * i)) * XXH_P32_1;
    }
}

/* hash_long_internal_loop over a 192-byte secret. */
static void xxh_long_loop(uint64_t *acc, const uint8_t *in, uint64_t len, const uint8_t *sec) {
    uint64_t stripes = (XXH_SECRET_SIZE - XXH_STRIPE_LEN) / XXH_CONSUME_RATE;
    uint64_t block = XXH_STRIPE_LEN * stripes, blocks = (len - 1) / block;
    for (uint64_t b = 0; b < blocks; b++) {
        for (uint64_t s = 0; s < stripes; s++) xxh_accumulate512(acc, in + b * block + s * XXH_STRIPE_LEN, sec + s * XXH_CONSUME_RATE);
        xxh_scramble(acc, sec + XXH_SECRET_SIZE - XXH_STRIPE_LEN);
    }
    uint64_t last = ((len - 1) - block * blocks) / XXH_STRIPE_LEN;
    for (uint64_t s = 0; s < last; s++) xxh_accumulate512(acc, in + blocks * block + s * XXH_STRIPE_LEN, sec + s * XXH_CONSUME_RATE);
    xxh_accumulate512(acc, in + len - XXH_STRIPE_LEN, sec + XXH_SECRET_SIZE - XXH_STRIPE_LEN - XXH_LASTACC_START);
}

static uint64_t xxh_merge(const uint64_t *acc, const uint8_t *sec, uint64_t start) {
    uint64_t r = start;
    for (int i = 0; i < 4; i++) r += xxh_fold64(acc[2 * i] ^ xxh_r64(sec + 16 * i), acc[2 * i + 1] ^ xxh_r64(sec + 16 * i + 8));
    return xxh3_avalanche(r);
}

static void xxh_long_acc(uint64_t *acc, const uint8_t *in, uint64_t len, uint64_t seed, uint8_t *secret_buf, const uint8_t **sec) {
    static const uint64_t init[8] = {XXH_P32_3, XXH_P64_1, XXH_P64_2, XXH_P64_3, XXH_P64_4, XXH_P32_2, XXH_P64_5, XXH_P32_1};
    for (int i = 0; i < 8; i++) acc[i] = init[i];
    *sec = xxh_default_secret;
    if (seed != 0) {
        xxh_seeded_secret(seed, secret_buf);
        *sec = secret_buf;
    }
    xxh_long_loop(acc, in, len, *sec);
}

/* XXH3 64-bit of `len` bytes at `p` (seed 0 is xxh3_64). */
uint64_t lust_xxh3_64(const uint8_t *p, uint64_t len, uint64_t seed) {
    const uint8_t *s = xxh_default_secret;
    if (len == 0) return xxh64_avalanche(seed ^ (xxh_r64(s + 56) ^ xxh_r64(s + 64)));
    if (len <= 3) {
        uint32_t combo = (uint32_t)p[0] << 16 | (uint32_t)p[len >> 1] << 24 | (uint32_t)p[len - 1] | (uint32_t)len << 8;
        uint64_t flip = (uint64_t)(xxh_r32(s) ^ xxh_r32(s + 4)) + seed;
        return xxh64_avalanche((uint64_t)combo ^ flip);
    }
    if (len <= 8) {
        uint64_t sd = seed ^ (uint64_t)xxh_swap32((uint32_t)seed) << 32;
        uint64_t flip = (xxh_r64(s + 8) ^ xxh_r64(s + 16)) - sd;
        uint64_t in64 = (uint64_t)xxh_r32(p + len - 4) + ((uint64_t)xxh_r32(p) << 32);
        return xxh_strong_avalanche(in64 ^ flip, len);
    }
    if (len <= 16) {
        uint64_t flip1 = (xxh_r64(s + 24) ^ xxh_r64(s + 32)) + seed, flip2 = (xxh_r64(s + 40) ^ xxh_r64(s + 48)) - seed;
        uint64_t lo = xxh_r64(p) ^ flip1, hi = xxh_r64(p + len - 8) ^ flip2;
        return xxh3_avalanche(len + xxh_swap64(lo) + hi + xxh_fold64(lo, hi));
    }
    if (len <= 128) {
        uint64_t acc = len * XXH_P64_1;
        if (len > 32) {
            if (len > 64) {
                if (len > 96) {
                    acc += xxh_mix16(p + 48, s + 96, seed);
                    acc += xxh_mix16(p + len - 64, s + 112, seed);
                }
                acc += xxh_mix16(p + 32, s + 64, seed);
                acc += xxh_mix16(p + len - 48, s + 80, seed);
            }
            acc += xxh_mix16(p + 16, s + 32, seed);
            acc += xxh_mix16(p + len - 32, s + 48, seed);
        }
        acc += xxh_mix16(p, s, seed);
        acc += xxh_mix16(p + len - 16, s + 16, seed);
        return xxh3_avalanche(acc);
    }
    if (len <= 240) {
        uint64_t acc = len * XXH_P64_1, rounds = len / 16, i = 0;
        for (; i < 8; i++) acc += xxh_mix16(p + 16 * i, s + 16 * i, seed);
        acc = xxh3_avalanche(acc);
        for (; i < rounds; i++) acc += xxh_mix16(p + 16 * i, s + 16 * (i - 8) + 3, seed);
        acc += xxh_mix16(p + len - 16, s + XXH_SECRET_SIZE_MIN - 17, seed);
        return xxh3_avalanche(acc);
    }
    uint64_t acc[8];
    uint8_t buf[XXH_SECRET_SIZE];
    const uint8_t *sec;
    xxh_long_acc(acc, p, len, seed, buf, &sec);
    return xxh_merge(acc, sec + XXH_MERGEACCS_START, len * XXH_P64_1);
}

/* XXH3 128-bit of `len` bytes at `p` (seed 0 is xxh3_128). */
lust_u128 lust_xxh3_128(const uint8_t *p, uint64_t len, uint64_t seed) {
    const uint8_t *s = xxh_default_secret;
    lust_u128 r;
    if (len == 0) {
        r.lo = xxh64_avalanche(seed ^ (xxh_r64(s + 64) ^ xxh_r64(s + 72)));
        r.hi = xxh64_avalanche(seed ^ (xxh_r64(s + 80) ^ xxh_r64(s + 88)));
        return r;
    }
    if (len <= 3) {
        uint32_t lo32 = (uint32_t)p[0] << 16 | (uint32_t)p[len >> 1] << 24 | (uint32_t)p[len - 1] | (uint32_t)len << 8;
        uint32_t hi32 = xxh_rotl32(xxh_swap32(lo32), 13);
        uint64_t flip_lo = ((uint64_t)xxh_r32(s) ^ xxh_r32(s + 4)) + seed, flip_hi = ((uint64_t)xxh_r32(s + 8) ^ xxh_r32(s + 12)) - seed;
        r.lo = xxh64_avalanche((uint64_t)lo32 ^ flip_lo);
        r.hi = xxh64_avalanche((uint64_t)hi32 ^ flip_hi);
        return r;
    }
    if (len <= 8) {
        uint64_t sd = seed ^ (uint64_t)xxh_swap32((uint32_t)seed) << 32;
        uint64_t in64 = (uint64_t)xxh_r32(p) + ((uint64_t)xxh_r32(p + len - 4) << 32);
        uint64_t flip = (xxh_r64(s + 16) ^ xxh_r64(s + 24)) + sd, lo, hi;
        xxh_mul128(in64 ^ flip, XXH_P64_1 + (len << 2), &lo, &hi);
        hi += lo << 1;
        lo ^= hi >> 3;
        lo = xxh_xorshift(lo, 35) * 0x9FB21C651E98DF25ull;
        r.lo = xxh_xorshift(lo, 28);
        r.hi = xxh3_avalanche(hi);
        return r;
    }
    if (len <= 16) {
        uint64_t flip_lo = (xxh_r64(s + 32) ^ xxh_r64(s + 40)) - seed, flip_hi = (xxh_r64(s + 48) ^ xxh_r64(s + 56)) + seed;
        uint64_t in_lo = xxh_r64(p), in_hi = xxh_r64(p + len - 8), m_lo, m_hi, r_lo, r_hi;
        xxh_mul128(in_lo ^ in_hi ^ flip_lo, XXH_P64_1, &m_lo, &m_hi);
        m_lo += (len - 1) << 54;
        in_hi ^= flip_hi;
        m_hi += in_hi + (uint64_t)(uint32_t)in_hi * (XXH_P32_2 - 1);
        m_lo ^= xxh_swap64(m_hi);
        xxh_mul128(m_lo, XXH_P64_2, &r_lo, &r_hi);
        r_hi += m_hi * XXH_P64_2;
        r.lo = xxh3_avalanche(r_lo);
        r.hi = xxh3_avalanche(r_hi);
        return r;
    }
    if (len <= 240) {
        uint64_t lo = len * XXH_P64_1, hi = 0;
        if (len <= 128) {
            if (len > 32) {
                if (len > 64) {
                    if (len > 96) xxh_mix32(&lo, &hi, p + 48, p + len - 64, s + 96, seed);
                    xxh_mix32(&lo, &hi, p + 32, p + len - 48, s + 64, seed);
                }
                xxh_mix32(&lo, &hi, p + 16, p + len - 32, s + 32, seed);
            }
            xxh_mix32(&lo, &hi, p, p + len - 16, s, seed);
        } else {
            uint64_t rounds = len / 32, i = 0;
            for (; i < 4; i++) xxh_mix32(&lo, &hi, p + 32 * i, p + 32 * i + 16, s + 32 * i, seed);
            lo = xxh3_avalanche(lo);
            hi = xxh3_avalanche(hi);
            for (; i < rounds; i++) xxh_mix32(&lo, &hi, p + 32 * i, p + 32 * i + 16, s + 3 + 32 * (i - 4), seed);
            xxh_mix32(&lo, &hi, p + len - 16, p + len - 32, s + XXH_SECRET_SIZE_MIN - 17 - 16, 0 - seed);
        }
        r.lo = xxh3_avalanche(lo + hi);
        r.hi = 0 - xxh3_avalanche(lo * XXH_P64_1 + hi * XXH_P64_4 + (len - seed) * XXH_P64_2);
        return r;
    }
    uint64_t acc[8];
    uint8_t buf[XXH_SECRET_SIZE];
    const uint8_t *sec;
    xxh_long_acc(acc, p, len, seed, buf, &sec);
    r.lo = xxh_merge(acc, sec + XXH_MERGEACCS_START, len * XXH_P64_1);
    r.hi = xxh_merge(acc, sec + XXH_SECRET_SIZE - 64 - XXH_MERGEACCS_START, ~(len * XXH_P64_2));
    return r;
}
