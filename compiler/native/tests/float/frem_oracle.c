/* Oracle check for the runtime float remainder (lust_fmod_f64/f32 in
   src/runtime/core.c): compares against host libm fmod over directed
   edges and 4M random bit patterns (both-NaN counts as a pass, the
   payload is free). Build and run with either compiler:
     gcc -O1 -w -o /tmp/frem tests/float/frem_oracle.c -lm && /tmp/frem
     tcc -o /tmp/frem tests/float/frem_oracle.c -lm && /tmp/frem     */
#define LUST_IMPL 1
#include "../../src/runtime/runtime.h"
#include "../../src/runtime/core.c"
#include <stdio.h>
#include <math.h>
#include <string.h>
static uint64_t rng = 0x123456789abcdef0ULL;
static uint64_t nxt(void) { rng ^= rng << 13; rng ^= rng >> 7; rng ^= rng << 17; return rng; }
static double d64(uint64_t u) { double d; memcpy(&d, &u, 8); return d; }
static float f32(uint32_t u) { float f; memcpy(&f, &u, 4); return f; }
static uint64_t b64(double d) { uint64_t u; memcpy(&u, &d, 8); return u; }
static uint32_t b32(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }
int main(void) {
    long bad = 0, n = 0;
    /* directed edges */
    double edges[][2] = {{5.5,2.0},{-5.5,2.0},{5.5,-2.0},{-5.5,-2.0},{0.0,3.0},{-0.0,3.0},{1e300,3.7},{1e-300,1e300},{123456789.0,0.1},{2.0,0.5},{7.0,7.0},{-7.0,7.0}};
    for (unsigned i = 0; i < sizeof(edges)/sizeof(edges[0]); i++) {
        double a = edges[i][0], b = edges[i][1];
        if (b64(lust_fmod_f64(a,b)) != b64(fmod(a,b))) { printf("edge64 %g %g: %g vs %g\n", a,b,lust_fmod_f64(a,b),fmod(a,b)); bad++; }
        n++;
    }
    for (long i = 0; i < 2000000; i++) {
        uint64_t ua = nxt(), ub = nxt();
        /* bias exponents toward finite mid-range half the time */
        if (i & 1) { ua = (ua & ~(0x7FFULL<<52)) | ((uint64_t)(nxt()%2046+1) << 52); ub = (ub & ~(0x7FFULL<<52)) | ((uint64_t)(nxt()%2046+1) << 52); }
        double a = d64(ua), b = d64(ub);
        double r1 = lust_fmod_f64(a,b), r2 = fmod(a,b);
        if (r1 != r1 && r2 != r2) { n++; continue; } /* both NaN: payload free */
        if (b64(r1) != b64(r2)) { if (bad < 5) printf("f64 bits %llx %llx: %llx vs %llx\n",(unsigned long long)ua,(unsigned long long)ub,(unsigned long long)b64(r1),(unsigned long long)b64(r2)); bad++; }
        float fa = f32((uint32_t)ua), fb = f32((uint32_t)ub);
        float s1 = lust_fmod_f32(fa,fb), s2 = fmodf(fa,fb);
        if (s1 != s1 && s2 != s2) { n++; continue; }
        if (b32(s1) != b32(s2)) { if (bad < 5) printf("f32 bits %x %x: %x vs %x\n",(uint32_t)ua,(uint32_t)ub,b32(s1),b32(s2)); bad++; }
        n += 2;
    }
    printf("cases %ld bad %ld\n", n, bad);
    return bad != 0;
}
