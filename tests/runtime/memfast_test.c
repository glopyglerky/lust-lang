/* tests/runtime/memfast_test.c (scripts/test-memfast.sh): the hand-written vector code of
   the release runtime against byte-by-byte references, built with ASan and UBSan, so a read
   outside a given range fails. memfast.c (memcpy, memmove, memcmp, bcmp, memchr): every length
   0..300 and some large ones, every source and destination offset 0..15, memmove overlaps
   -80..80, memcmp with the difference at each index, memchr hits at each index and misses.
   deps.c lust_memmem: 240,000 random cases against a naive search; lust_memchr3 and
   lust_memrchr3: 60,000 against byte loops. -DMF_NO_V16 tests the
   word path that non-x86_64 targets take. */
#include <stdint.h>
typedef struct { uint64_t lo, hi; } lust_u128;
#include "../../src/runtime/deps.c"

#define memcpy mf_memcpy
#define memmove mf_memmove
#define memcmp mf_memcmp
#define bcmp mf_bcmp
#define memchr mf_memchr
#include "../../src/runtime/memfast.c"
#undef memcpy
#undef memmove
#undef memcmp
#undef bcmp
#undef memchr
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static unsigned rng = 12345;
static unsigned rnd(void) { rng = rng * 1103515245u + 12345u; return rng >> 8; }
static int fails = 0;
#define FAIL(...) do { if (fails++ < 20) { printf(__VA_ARGS__); printf("\n"); } } while (0)

/* An exact-size heap block per test, so ASan sees one byte past the end. */
static unsigned char *blk(size_t n) { unsigned char *p = malloc(n ? n : 1); for (size_t i = 0; i < n; i++) p[i] = (unsigned char)rnd(); return p; }

static int sgn(int x) { return (x > 0) - (x < 0); }
static int refcmp(const unsigned char *a, const unsigned char *b, size_t n) { for (size_t i = 0; i < n; i++) if (a[i] != b[i]) return (int)a[i] - (int)b[i]; return 0; }

static int mf_main(void) {
    size_t lens[400]; int nl = 0;
    for (size_t n = 0; n <= 300; n++) lens[nl++] = n;
    size_t big[] = {511, 512, 513, 1000, 4095, 4096, 4097, 65536 + 7};
    for (int i = 0; i < 8; i++) lens[nl++] = big[i];
    for (int li = 0; li < nl; li++) {
        size_t n = lens[li];
        for (int so = 0; so < 16; so++) for (int dof = 0; dof < 16; dof += (n > 600 ? 5 : 1)) {
            unsigned char *s = blk(n + so), *d = blk(n + dof), *r = malloc(n + dof + 1);
            memcpy(r, d, n + dof);
            void *ret = mf_memcpy(d + dof, s + so, n);
            memcpy(r + dof, s + so, n);
            if (ret != d + dof || memcmp(d, r, n + dof)) FAIL("memcpy n=%zu so=%d do=%d", n, so, dof);
            free(s); free(d); free(r);
        }
        /* memmove: one buffer, shifts -80..80 */
        for (int sh = -80; sh <= 80; sh += (n > 600 ? 7 : 1)) {
            size_t pad = 80, tot = n + 2 * pad;
            unsigned char *b = blk(tot), *r = malloc(tot);
            memcpy(r, b, tot);
            unsigned char *src = b + pad, *dst = b + pad + sh;
            void *ret = mf_memmove(dst, src, n);
            memmove(r + pad + sh, r + pad, n);
            if (ret != dst || memcmp(b, r, tot)) FAIL("memmove n=%zu sh=%d", n, sh);
            free(b); free(r);
        }
        /* memcmp / bcmp: equal, and one difference at each index (both signs) */
        for (int off = 0; off < 4; off++) {
            unsigned char *a = blk(n + off), *c = malloc(n + off + 1);
            memcpy(c, a, n + off);
            if (mf_memcmp(a + off, c + off, n) != 0 || mf_bcmp(a + off, c + off, n) != 0) FAIL("memcmp equal n=%zu", n);
            for (size_t k = 0; k < n; k += (n > 600 ? 97 : 1)) {
                unsigned char save = c[off + k];
                c[off + k] = (unsigned char)(save + 1 + rnd() % 255);
                int e = refcmp(a + off, c + off, n), g = mf_memcmp(a + off, c + off, n);
                if (e != g || !mf_bcmp(a + off, c + off, n)) FAIL("memcmp n=%zu k=%zu want %d got %d", n, k, e, g);
                if (refcmp(c + off, a + off, n) != mf_memcmp(c + off, a + off, n)) FAIL("memcmp rev n=%zu k=%zu", n, k);
                c[off + k] = save;
            }
            free(a); free(c);
        }
        /* memchr: a miss, then a hit at each index */
        for (int off = 0; off < 4; off++) {
            unsigned char *a = malloc(n + off + 1);
            for (size_t i = 0; i < n + off; i++) a[i] = (unsigned char)(1 + rnd() % 200);
            if (mf_memchr(a + off, 0, n) != NULL) FAIL("memchr miss n=%zu", n);
            if (mf_memchr(a + off, 255 + 256, n) != memchr(a + off, 255 + 256, n)) FAIL("memchr int arg n=%zu", n);
            for (size_t k = 0; k < n; k += (n > 600 ? 61 : 1)) {
                unsigned char save = a[off + k];
                a[off + k] = 0;
                if (mf_memchr(a + off, 0, n) != memchr(a + off, 0, n)) FAIL("memchr n=%zu k=%zu", n, k);
                if (mf_memchr(a + off, 0, k) != NULL) FAIL("memchr prefix n=%zu k=%zu", n, k);
                a[off + k] = save;
            }
            for (int c = 0; c < 256; c += 37)
                if (mf_memchr(a + off, c, n) != memchr(a + off, c, n)) FAIL("memchr c=%d n=%zu", c, n);
            free(a);
        }
    }
    printf("memfast: %d failures\n", fails);
    return fails != 0;
}

static uint64_t naive(const uint8_t *h, uint64_t hl, const uint8_t *n, uint64_t nl) {
    if (nl == 0) return 0;
    for (uint64_t i = 0; i + nl <= hl; i++) if (memcmp(h + i, n, nl) == 0) return i;
    return UINT64_MAX;
}
static int mm_run(uint64_t hmax, uint64_t nmax, int iters) {
    int fails = 0;
    for (int it = 0; it < iters; it++) {
        uint64_t hl = rnd() % hmax, nl = rnd() % nmax;
        int alpha = 1 + rnd() % 6;
        uint8_t *h = malloc(hl ? hl : 1), *n = malloc(nl ? nl : 1);
        for (uint64_t i = 0; i < hl; i++) h[i] = (uint8_t)("ab e\n/x"[rnd() % alpha] + (rnd() % 50 == 0));
        for (uint64_t i = 0; i < nl; i++) n[i] = (uint8_t)("ab e\n/x"[rnd() % alpha]);
        if (nl && hl > nl && rnd() % 2) { uint64_t at = rnd() % (hl - nl + 1); memcpy(h + at, n, nl); }
        uint64_t a = lust_memmem(h, hl, n, nl), b = naive(h, hl, n, nl);
        if (a != b && fails++ < 10) printf("memmem hl=%lu nl=%lu got %lu want %lu\n", (unsigned long)hl, (unsigned long)nl, (unsigned long)a, (unsigned long)b);
        free(h); free(n);
    }
    return fails;
}

/* lust_memchr3 / lust_memrchr3 against byte loops: lengths 0-200 and some long ones, the
   needles (two of them equal, as memchr2 passes) at random spots, every start offset. */
static int m3_run(int iters) {
    int fails = 0;
    for (int it = 0; it < iters; it++) {
        uint64_t n = it < 4000 ? (uint64_t)(it % 200) : 200 + rnd() % 3000, off = rnd() % 16;
        uint8_t *buf = malloc(n + off + 1), *h = buf + off;
        for (uint64_t i = 0; i < n; i++) h[i] = (uint8_t)("abcdefgh"[rnd() % 8]);
        uint8_t a = (uint8_t)"\n\r\xe2x"[rnd() % 4], b = (uint8_t)"\n\r\xe2y"[rnd() % 4], c = rnd() % 3 ? b : (uint8_t)'z';
        for (int k = rnd() % 4; k > 0 && n; k--) h[rnd() % n] = (uint8_t)"\n\r\xe2xyz"[rnd() % 6];
        uint64_t want = UINT64_MAX, rwant = UINT64_MAX;
        for (uint64_t i = 0; i < n; i++)
            if (h[i] == a || h[i] == b || h[i] == c) { if (want == UINT64_MAX) want = i; rwant = i; }
        uint64_t got = lust_memchr3(a, b, c, h, n), rgot = lust_memrchr3(a, b, c, h, n);
        if ((got != want || rgot != rwant) && fails++ < 10)
            printf("memchr3 n=%lu: got %lu/%lu want %lu/%lu\n", (unsigned long)n, (unsigned long)got, (unsigned long)rgot, (unsigned long)want, (unsigned long)rwant);
        free(buf);
    }
    return fails;
}

int main(void) {
    int f = mf_main();
    int m = mm_run(200, 12, 200000) + mm_run(3000, 48, 40000);
    printf("memmem: %d failures\n", m);
    int m3 = m3_run(60000);
    printf("memchr3: %d failures\n", m3);
    return f || m || m3;
}
