// K3: check u128.h against vectors from u128_oracle.rs (stdin or arg file).
// Vector line: alo ahi blo bhi sh <add> <sub> <mul> <udiv> <urem> <idiv> <irem>
//              <shl> <ushr> <sshr> ucmp icmp ufmt ifmt   (pairs are lo hi hex)
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "u128.h"
static uint64_t rd(char **p) { return strtoull(*p, p, 16); }
static lust_u128 rd128(char **p) { lust_u128 v; v.lo = rd(p); v.hi = rd(p); return v; }
static int eq(lust_u128 a, lust_u128 b) { return a.lo == b.lo && a.hi == b.hi; }
int main(int argc, char **argv) {
    FILE *f = argc > 1 ? fopen(argv[1], "r") : stdin;
    if (!f) { printf("FAIL open vectors\n"); return 1; }
    char line[1024]; long n = 0, bad = 0;
    int bad_add = 0, bad_sub = 0, bad_mul = 0, bad_udiv = 0, bad_urem = 0, bad_idiv = 0,
        bad_irem = 0, bad_sh = 0, bad_cmp = 0, bad_fmt = 0;
    while (fgets(line, sizeof line, f)) {
        char *p = line;
        lust_u128 a = rd128(&p), b = rd128(&p);
        unsigned sh = (unsigned)strtoul(p, &p, 10);
        lust_u128 e_add = rd128(&p), e_sub = rd128(&p), e_mul = rd128(&p);
        lust_u128 e_udiv = rd128(&p), e_urem = rd128(&p), e_idiv = rd128(&p), e_irem = rd128(&p);
        lust_u128 e_shl = rd128(&p), e_ushr = rd128(&p), e_sshr = rd128(&p);
        long e_ucmp = strtol(p, &p, 10), e_icmp = strtol(p, &p, 10);
        char e_ufmt[64], e_ifmt[64];
        if (sscanf(p, "%63s %63s", e_ufmt, e_ifmt) != 2) { bad++; continue; }
        n++;
        if (!eq(u128_add(a, b), e_add)) bad_add++;
        if (!eq(u128_sub(a, b), e_sub)) bad_sub++;
        if (!eq(u128_mul(a, b), e_mul)) bad_mul++;
        if (!(b.lo == 0 && b.hi == 0)) {
            if (!eq(u128_div(a, b), e_udiv)) bad_udiv++;
            if (!eq(u128_rem(a, b), e_urem)) bad_urem++;
            // Rust i128 MIN / -1 panics; the oracle writes the wrapped pair for
            // that case and C long division wraps the same way, so compare all.
            if (!eq(i128_div(a, b), e_idiv)) bad_idiv++;
            if (!eq(i128_rem(a, b), e_irem)) bad_irem++;
        }
        if (!eq(u128_shl(a, sh), e_shl) || !eq(u128_shr(a, sh), e_ushr) || !eq(i128_sar(a, sh), e_sshr)) bad_sh++;
        if (u128_cmp(a, b) != e_ucmp || i128_cmp(a, b) != e_icmp) bad_cmp++;
        char buf[64]; u128_format(a, buf);
        char buf2[64]; i128_format(a, buf2);
        if (strcmp(buf, e_ufmt) != 0 || strcmp(buf2, e_ifmt) != 0) bad_fmt++;
    }
    bad = bad_add + bad_sub + bad_mul + bad_udiv + bad_urem + bad_idiv + bad_irem + bad_sh + bad_cmp + bad_fmt;
    if (n == 0) { printf("FAIL u128_no_vectors\nRESULT FAIL fails=1\n"); return 1; }
    printf("INFO u128 vectors=%ld bad_add=%d bad_sub=%d bad_mul=%d bad_udiv=%d bad_urem=%d bad_idiv=%d bad_irem=%d bad_shifts=%d bad_cmp=%d bad_fmt=%d\n",
           n, bad_add, bad_sub, bad_mul, bad_udiv, bad_urem, bad_idiv, bad_irem, bad_sh, bad_cmp, bad_fmt);
    printf("INFO u128_layout size=%zu align=%zu\n", sizeof(lust_u128),
#ifdef __STDC_VERSION__
           _Alignof(lust_u128)
#else
           (size_t)8
#endif
    );
    printf("%s u128_vs_rustc\nRESULT %s fails=%ld\n", bad ? "FAIL" : "PASS", bad ? "FAIL" : "PASS", bad);
    return bad != 0;
}
