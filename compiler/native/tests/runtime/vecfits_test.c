/* runtime.h's lust_vec_fits (RawVec's size rule as one multiply) against the division it
   replaced, n <= isize::MAX / elem: element sizes 1, 2, 3, large odd ones and sizes near and
   over isize::MAX, each with n at 0, 1, M/elem - 1, M/elem, M/elem + 1 and values whose
   n * elem wraps, then random pairs near the boundary. A zero-sized element never comes
   here (the Vec core handles it first; tests/unwind/vec_capacity_boundary.rs). Built by
   scripts/test-memfast.sh with each C compiler and with tcc (whose form is the division). */
#include "../../src/runtime/runtime.h"
#include <stdio.h>

#define M 9223372036854775807ull
#define U 18446744073709551615ull
static uint64_t rng = 0x9e3779b97f4a7c15ull;
static uint64_t rnd(void) {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    return rng;
}
static long cases, fails;
static void check(uint64_t n, uint64_t e) {
    int want = n <= M / e, got = lust_vec_fits(n, e);
    cases++;
    if (got != want && fails++ < 10)
        printf("lust_vec_fits(%llu, %llu) = %d, want %d\n", (unsigned long long)n, (unsigned long long)e, got, want);
}

int main(void) {
    static const uint64_t elems[] = {1, 2, 3, 5, 7, 8, 24, 1000003, 3037000499ull, 4294967295ull, 4294967297ull,
                                     M / 3, M / 2, M / 2 + 1, M - 1, M, M + 1, U};
    for (unsigned i = 0; i < sizeof elems / sizeof elems[0]; i++) {
        uint64_t e = elems[i], q = M / e;
        uint64_t ns[] = {0, 1, 2, q ? q - 1 : 0, q, q + 1, q + 2, U / e, U / e + 1, 1ull << 32, 1ull << 63, M, M + 1, U};
        for (unsigned j = 0; j < sizeof ns / sizeof ns[0]; j++) check(ns[j], elems[i]);
    }
    /* A literal elem: the folded compare. */
    for (int k = -3; k <= 3; k++) {
#define LIT(e)                                                                                 \
    do {                                                                                       \
        uint64_t n = M / (e) + (uint64_t)k;                                                    \
        int want = n <= M / (e), got = lust_vec_fits(n, (e));                                  \
        cases++;                                                                               \
        if (got != want && fails++ < 10) printf("literal %llu: n %llu got %d\n", (unsigned long long)(e), (unsigned long long)n, got); \
    } while (0)
        LIT(1); LIT(3); LIT(24); LIT(1000003);
    }
    for (int k = 0; k < 1000000; k++) {
        uint64_t e = rnd() >> (rnd() % 64);
        if (!e) e = 1;
        uint64_t q = M / e, n = rnd() % 4 ? q + rnd() % 5 - 2 : rnd() >> (rnd() % 64);
        check(n, e);
    }
    printf("vecfits: %ld cases, %ld failures\n", cases, fails);
    return fails != 0;
}
