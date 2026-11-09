/* Case generator for the float fuzz gate. One case per line on stdout.
   Modes: bits64, bits32, spec64, spec32, dec, ints. The same generator
   output feeds the Rust oracle and the C harness; fuzz.sh diffs them. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static uint64_t st;
static uint64_t rnd(void) {
    uint64_t z = (st += 0x9E3779B97F4A7C15ull);
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ull;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBull;
    return z ^ (z >> 31);
}
static uint64_t pick(uint64_t n) { return n ? rnd() % n : 0; }

static double dfrom(uint64_t b) { double d; memcpy(&d, &b, 8); return d; }
static uint64_t dbits(double d) { uint64_t b; memcpy(&b, &d, 8); return b; }
static uint32_t fbits(float f) { uint32_t b; memcpy(&b, &f, 4); return b; }

/* Random f64 bit patterns with biased classes around hard regions. */
static uint64_t gen64(void) {
    uint64_t r = rnd();
    switch (pick(16)) {
    case 0: case 1: case 2: case 3:
        return r;
    case 4: /* exponent near 1.0 */
        return (r & ((1ull << 52) - 1)) | ((1023 - 40 + pick(81)) << 52) | (pick(2) << 63);
    case 5: /* subnormal or zero */
        return (r & ((1ull << 52) - 1)) | (pick(2) << 63);
    case 6: /* binade edges */
        return ((uint64_t)(1 + pick(2045)) << 52) | (pick(2) ? pick(4) : ((1ull << 52) - 1 - pick(4))) | (pick(2) << 63);
    case 7: { /* trailing zero mantissa bits: short exact decimals, ties */
        int k = (int)pick(53);
        return r & (~0ull << k);
    }
    case 8: { /* small integers and binary fractions */
        int64_t v = (int64_t)rnd() % 2000000000 - 1000000000;
        double d = (double)v / (double)(1ull << pick(20));
        return dbits(d);
    }
    case 9: { /* near a short decimal (parsed by glibc strtod) */
        char buf[64];
        snprintf(buf, sizeof buf, "%llu.%03llue%lld", (unsigned long long)pick(100000), (unsigned long long)pick(1000),
                 (long long)((int)pick(640) - 320));
        uint64_t b = dbits(strtod(buf, NULL));
        return b + pick(5) - 2;
    }
    case 10: /* inf, NaN payloads */
        return (r & ((1ull << 52) - 1)) | (0x7FFull << 52) | (pick(2) << 63);
    case 11: /* near MAX */
        return 0x7FEFFFFFFFFFFFFFull - pick(100000);
    case 12: /* near MIN_POSITIVE */
        return 0x0010000000000000ull + pick(200000) - 100000;
    case 13: { /* integers near 2^53 */
        double d = (double)(9007199254740992ll + (int64_t)pick(100000) - 50000);
        return dbits(pick(2) ? -d : d);
    }
    case 14: { /* tiny powers of two and neighbors */
        uint64_t b = (uint64_t)(1 + pick(2046)) << 52;
        return b + pick(5) - 2;
    }
    default:
        return r;
    }
}

static uint32_t gen32(void) {
    uint64_t r = rnd();
    switch (pick(12)) {
    case 0: case 1: case 2:
        return (uint32_t)r;
    case 3:
        return ((uint32_t)r & 0x7FFFFF) | ((127 - 20 + (uint32_t)pick(41)) << 23) | ((uint32_t)pick(2) << 31);
    case 4:
        return ((uint32_t)r & 0x7FFFFF) | ((uint32_t)pick(2) << 31);
    case 5: {
        int k = (int)pick(24);
        return (uint32_t)r & (~0u << k);
    }
    case 6: {
        float f = (float)((int64_t)rnd() % 2000000 - 1000000) / (float)(1u << pick(12));
        return fbits(f);
    }
    case 7: {
        char buf[64];
        snprintf(buf, sizeof buf, "%llu.%02llue%lld", (unsigned long long)pick(10000), (unsigned long long)pick(100),
                 (long long)((int)pick(90) - 45));
        return fbits(strtof(buf, NULL)) + (uint32_t)pick(5) - 2;
    }
    case 8:
        return ((uint32_t)r & 0x7FFFFF) | (0xFFu << 23) | ((uint32_t)pick(2) << 31);
    case 9:
        return 0x7F7FFFFF - (uint32_t)pick(100000);
    case 10:
        return 0x00800000 + (uint32_t)pick(2000) - 1000;
    default:
        return (uint32_t)r;
    }
}

/* One random decimal string (mostly valid, some grammar fuzz). */
static void gen_dec(char *buf, size_t cap) {
    int cls = (int)pick(10);
    char *p = buf, *end = buf + cap - 1;
    if (cls == 9) {
        static const char alpha[] = "0123456789.eE+-infaNty_ xINF";
        int n = (int)pick(12), i;
        for (i = 0; i < n && p < end; i++) *p++ = alpha[pick(sizeof alpha - 1)];
        *p = 0;
        return;
    }
    if (pick(3) == 0) *p++ = pick(2) ? '-' : '+';
    if (cls == 8) {
        static const char *words[] = {"inf", "INF", "infinity", "Infinity", "NaN", "nan", "NAN", "infi", "nanx", "", ".", "e5", ".e5"};
        const char *w = words[pick(sizeof words / sizeof *words)];
        while (*w && p < end) *p++ = *w++;
        *p = 0;
        return;
    }
    {
        int ndig, i, point, want_exp, expv;
        switch (cls) {
        case 0: ndig = 1 + (int)pick(17); break;
        case 1: ndig = 18 + (int)pick(45); break;
        case 2: ndig = 300 + (int)pick(500); break;
        case 3: ndig = 750 + (int)pick(400); break;
        case 4: ndig = 1 + (int)pick(25); break;
        case 5: ndig = 15 + (int)pick(10); break;
        default: ndig = 1 + (int)pick(19); break;
        }
        point = pick(4) == 0 ? -1 : (int)pick(ndig + 1);
        for (i = 0; i < ndig && p < end; i++) {
            if (i == point) *p++ = '.';
            if (pick(10) == 0)
                *p++ = '0';
            else
                *p++ = (char)('0' + pick(10));
        }
        if (point == ndig && p < end) *p++ = '.';
        if (cls == 4) {
            /* halfway-shaped tail: digit 5 then only zeros */
            if (p + 8 < end) {
                int z;
                *p++ = '5';
                for (z = 0; z < (int)pick(6); z++) *p++ = '0';
            }
        }
        want_exp = pick(4) != 0;
        if (want_exp && p + 16 < end) {
            *p++ = pick(2) ? 'e' : 'E';
            switch (cls) {
            case 5: expv = 290 + (int)pick(30) - (pick(2) ? 620 : 0); break; /* near overflow edges */
            case 4: expv = (int)pick(60) - 330; break;                     /* subnormal region */
            case 6: expv = (int)pick(2000) - 1000; break;
            default: expv = (int)pick(660) - 330; break;
            }
            if (expv < 0) {
                *p++ = '-';
                expv = -expv;
            } else if (pick(3) == 0) {
                *p++ = '+';
            }
            p += snprintf(p, (size_t)(end - p), "%d", expv);
        }
        *p = 0;
    }
}

int main(int argc, char **argv) {
    const char *mode;
    long n, i;
    if (argc < 4) {
        fprintf(stderr, "usage: gen <mode> <count> <seed>\n");
        return 2;
    }
    mode = argv[1];
    n = atol(argv[2]);
    st = strtoull(argv[3], NULL, 10) * 0x9E3779B97F4A7C15ull + 1;
    setvbuf(stdout, NULL, _IOFBF, 1 << 20);
    for (i = 0; i < n; i++) {
        if (!strcmp(mode, "bits64")) {
            printf("%016llx\n", (unsigned long long)gen64());
        } else if (!strcmp(mode, "bits32")) {
            printf("%08x\n", gen32());
        } else if (!strcmp(mode, "spec64")) {
            printf("%016llx %llu\n", (unsigned long long)gen64(), (unsigned long long)pick(44));
        } else if (!strcmp(mode, "spec32")) {
            printf("%08x %llu\n", gen32(), (unsigned long long)pick(44));
        } else if (!strcmp(mode, "dec")) {
            char buf[2048];
            gen_dec(buf, sizeof buf);
            printf("%s\n", buf);
        } else if (!strcmp(mode, "ints")) {
            uint64_t lo = rnd(), hi = rnd();
            switch (pick(8)) {
            case 0: hi = 0; break;
            case 1: lo = (1ull << 53) + pick(4096) - 2048; hi = 0; break;
            case 2: lo = ~0ull - pick(4096); break;
            case 3: hi = 1ull << pick(64); lo = pick(4096); break;
            case 4: lo = 1ull << pick(64); hi = 0; break;
            case 5: lo &= (1ull << 24) - 1; hi = 0; break;
            default: break;
            }
            printf("%016llx %016llx\n", (unsigned long long)lo, (unsigned long long)hi);
        } else {
            fprintf(stderr, "bad mode %s\n", mode);
            return 2;
        }
    }
    return 0;
}
