// K3: time u128.h ops. Prints ns/op per op. The sink prevents dead-code removal
// only where the compiler tries; tcc does almost no such removal anyway.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "u128.h"
static uint64_t rng_state = 0x853c49e6748fea9bull;
static uint64_t next64(void) {
    rng_state ^= rng_state << 13; rng_state ^= rng_state >> 7; rng_state ^= rng_state << 17;
    return rng_state;
}
static double now_s(void) {
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec + ts.tv_nsec / 1e9;
}
#define NBUF 4096
static lust_u128 buf_a[NBUF], buf_b[NBUF];
static void bench(const char *name, int op, long iters) {
    uint64_t sink = 0; char sbuf[64];
    double t0 = now_s();
    for (long i = 0; i < iters; i++) {
        lust_u128 a = buf_a[i & (NBUF - 1)], b = buf_b[i & (NBUF - 1)], r;
        switch (op) {
            case 0: r = u128_add(a, b); break;
            case 1: r = u128_mul(a, b); break;
            case 2: r = u128_div(a, b.lo | 1 ? u128_make(b.lo | 1, b.hi) : b); break;
            case 3: r = u128_shl(a, (unsigned)(i & 127)); break;
            case 4: u128_format(a, sbuf); r = u128_make((uint64_t)strlen(sbuf), 0); break;
            default: r = u128_make((uint64_t)u128_cmp(a, b), 0); break;
        }
        sink += r.lo + r.hi;
    }
    double dt = now_s() - t0;
    printf("BENCH u128_%s ns_per_op=%.2f iters=%ld sink=%llu\n", name, dt * 1e9 / iters, iters,
           (unsigned long long)sink);
}
int main(int argc, char **argv) {
    long iters = argc > 1 ? atol(argv[1]) : 2000000;
    for (int i = 0; i < NBUF; i++) {
        uint64_t a0 = next64(), a1 = next64(), b0 = next64() | 1, b1 = next64();
        buf_a[i] = u128_make(a0, a1); buf_b[i] = u128_make(b0, b1);
    }
    bench("add", 0, iters);
    bench("mul", 1, iters);
    bench("div", 2, iters / 20 + 1);
    bench("shl", 3, iters);
    bench("format", 4, iters / 20 + 1);
    bench("cmp", 5, iters);
    return 0;
}
