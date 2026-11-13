/* C harness for the float runtime. Compiles the real runtime (including
   src/runtime/float.c) with the pinned tcc, the same compiler lust uses.
   Modes stream cases from stdin like oracle.rs does and must byte-match
   it; `edges` runs directed assertions; `bench` reports ns per call. */
#define LUST_IMPL 1
#include "../../src/runtime/runtime.h"
#include "../../src/runtime/core.c"
#include <stdio.h>
#include <time.h>

static char line_buf[65536];

/* Reads one case line, returns its length or -1 at end of input. Only the
   trailing newline is framing; spaces and tabs belong to the case. */
static int read_case(void) {
    size_t n;
    if (!fgets(line_buf, sizeof line_buf, stdin)) return -1;
    n = strlen(line_buf);
    if (n && line_buf[n - 1] == '\n') line_buf[--n] = 0;
    return (int)n;
}

static void emit_str(lust_String *s) { fwrite(s->ptr, 1, s->len, stdout); }

static lust_float_spec spec_of(uint8_t mode, uint32_t prec) {
    lust_float_spec s;
    memset(&s, 0, sizeof s);
    s.precision = prec;
    s.mode = mode;
    return s;
}
#define NP LUST_FLOAT_NO_PRECISION
/* The same spec list as oracle.rs. Field order: precision, width, fill,
   align, plus, zero_pad, alt, mode. Fill 0xE9 is 'é'. */
static const lust_float_spec spec_table[44] = {
    {0, 0, 0, 0, 0, 0, 0, 0},          /* 0  {:.0} */
    {1, 0, 0, 0, 0, 0, 0, 0},          /* 1  {:.1} */
    {2, 0, 0, 0, 0, 0, 0, 0},          /* 2  {:.2} */
    {5, 0, 0, 0, 0, 0, 0, 0},          /* 3  {:.5} */
    {10, 0, 0, 0, 0, 0, 0, 0},         /* 4  {:.10} */
    {17, 0, 0, 0, 0, 0, 0, 0},         /* 5  {:.17} */
    {30, 0, 0, 0, 0, 0, 0, 0},         /* 6  {:.30} */
    {2, 0, 0, 0, 0, 0, 0, 1},          /* 7  {:.2?} */
    {0, 0, 0, 0, 0, 0, 0, 1},          /* 8  {:.0?} */
    {0, 0, 0, 0, 0, 0, 0, 2},          /* 9  {:.0e} */
    {2, 0, 0, 0, 0, 0, 0, 2},          /* 10 {:.2e} */
    {6, 0, 0, 0, 0, 0, 0, 3},          /* 11 {:.6E} */
    {20, 0, 0, 0, 0, 0, 0, 2},         /* 12 {:.20e} */
    {NP, 12, 0, 0, 0, 0, 0, 0},        /* 13 {:12} */
    {NP, 12, 0, '<', 0, 0, 0, 0},      /* 14 {:<12} */
    {NP, 12, 0, '^', 0, 0, 0, 0},      /* 15 {:^12} */
    {NP, 13, 0, '^', 0, 0, 0, 0},      /* 16 {:^13} */
    {NP, 12, 0, 0, 0, 1, 0, 0},        /* 17 {:012} */
    {NP, 12, 0, 0, 1, 1, 0, 0},        /* 18 {:+012} */
    {NP, 12, 0, 0, 1, 0, 0, 0},        /* 19 {:+12} */
    {3, 12, 0, 0, 0, 1, 0, 0},         /* 20 {:012.3} */
    {2, 12, 0, '<', 0, 1, 0, 0},       /* 21 {:<012.2} */
    {NP, 12, '*', '>', 0, 0, 0, 0},    /* 22 {:*>12} */
    {NP, 12, '*', '<', 0, 0, 0, 0},    /* 23 {:*<12} */
    {NP, 13, '*', '^', 0, 0, 0, 0},    /* 24 {:*^13} */
    {NP, 12, '0', '>', 0, 0, 0, 0},    /* 25 {:0>12} */
    {NP, 12, '0', '<', 0, 0, 0, 0},    /* 26 {:0<12} */
    {NP, 0, 0, 0, 0, 0, 1, 0},         /* 27 {:#} */
    {2, 0, 0, 0, 0, 0, 1, 0},          /* 28 {:#.2} */
    {NP, 0, 0, 0, 0, 0, 1, 1},         /* 29 {:#?} */
    {4, 12, 0, 0, 0, 0, 0, 2},         /* 30 {:12.4e} */
    {2, 12, 0, 0, 1, 1, 0, 2},         /* 31 {:+012.2e} */
    {50, 0, 0, 0, 0, 0, 0, 0},         /* 32 {:.50} */
    {100, 0, 0, 0, 0, 0, 0, 2},        /* 33 {:.100e} */
    {NP, 10, 0xE9, '>', 0, 0, 0, 0},   /* 34 {:é>10} */
    {NP, 5, 0, 0, 0, 0, 0, 0},         /* 35 {:5} */
    {500, 0, 0, 0, 0, 0, 0, 0},        /* 36 {:.500} */
    {NP, 12, 0, '^', 0, 1, 0, 0},      /* 37 {:^012} */
    {NP, 0, 0, 0, 1, 0, 0, 0},         /* 38 {:+} */
    {NP, 0, 0, 0, 1, 0, 0, 1},         /* 39 {:+?} */
    {NP, 0, 0, 0, 1, 0, 0, 2},         /* 40 {:+e} */
    {NP, 12, 0, 0, 0, 1, 0, 2},        /* 41 {:012e} */
    {0, 12, 0, 0, 0, 0, 0, 0},         /* 42 {:12.0} */
    {NP, 12, 0, '^', 1, 0, 0, 0},      /* 43 {:^+12} */
};

/* Decimal printing for a 128-bit magnitude or signed value (hi, lo). */
static void print_u128(uint64_t lo, uint64_t hi) {
    char buf[48];
    int n = 0;
    if (lo == 0 && hi == 0) {
        putchar('0');
        return;
    }
    while (lo || hi) {
        /* divide the pair by 10 */
        uint64_t qhi = hi / 10, r = hi % 10;
        uint64_t cur = (r << 32) | (lo >> 32);
        uint64_t qmid = cur / 10;
        r = cur % 10;
        cur = (r << 32) | (lo & 0xFFFFFFFFu);
        buf[n++] = (char)('0' + cur % 10);
        lo = (qmid << 32) | (cur / 10);
        hi = qhi;
    }
    while (n) putchar(buf[--n]);
}
static void print_i128(uint64_t lo, uint64_t hi) {
    if ((int64_t)hi < 0) {
        uint64_t nlo = ~lo + 1, nhi = ~hi + (lo == 0 ? 1 : 0);
        putchar('-');
        print_u128(nlo, nhi);
    } else {
        print_u128(lo, hi);
    }
}

static int mode_stream(const char *mode) {
    int len;
    for (;;) {
        len = read_case();
        if (len < 0) return 0;
        if (!strcmp(mode, "fmt64") || !strcmp(mode, "fmt32")) {
            int is64 = mode[3] == '6';
            lust_String s = lust_string_new();
            int m;
            for (m = 0; m < 4; m++) {
                uint8_t modes[4] = {LUST_FLOAT_DISPLAY, LUST_FLOAT_DEBUG, LUST_FLOAT_LOWER_EXP, LUST_FLOAT_UPPER_EXP};
                if (m) lust_string_push_bytes(&s, "\t", 1);
                if (is64)
                    lust_fmt_f64(&s, lust_f64_from_bits(strtoull(line_buf, NULL, 16)), spec_of(modes[m], NP));
                else
                    lust_fmt_f32(&s, lust_f32_from_bits((uint32_t)strtoull(line_buf, NULL, 16)), spec_of(modes[m], NP));
            }
            emit_str(&s);
            putchar('\n');
            free(s.ptr);
        } else if (!strcmp(mode, "spec64") || !strcmp(mode, "spec32")) {
            int is64 = mode[4] == '6';
            char *sp = strchr(line_buf, ' ');
            int idx = atoi(sp + 1);
            lust_String s = lust_string_new();
            *sp = 0;
            if (is64)
                lust_fmt_f64(&s, lust_f64_from_bits(strtoull(line_buf, NULL, 16)), spec_table[idx]);
            else
                lust_fmt_f32(&s, lust_f32_from_bits((uint32_t)strtoull(line_buf, NULL, 16)), spec_table[idx]);
            emit_str(&s);
            putchar('\n');
            free(s.ptr);
        } else if (!strcmp(mode, "js64") || !strcmp(mode, "js32")) {
            char buf[40];
            uint64_t n = mode[2] == '6' ? lust_fmt_js_f64_buf(buf, lust_f64_from_bits(strtoull(line_buf, NULL, 16)))
                                          : lust_fmt_js_f32_buf(buf, lust_f32_from_bits((uint32_t)strtoull(line_buf, NULL, 16)));
            fwrite(buf, 1, n, stdout);
            putchar('\n');
        } else if (!strcmp(mode, "parse64")) {
            lust_str ls;
            double d = 0;
            int rc;
            ls.ptr = (uint8_t *)line_buf;
            ls.len = (uint64_t)len;
            rc = lust_parse_f64(ls, &d);
            printf("%d %016llx\n", rc, (unsigned long long)(rc ? 0 : lust_f64_bits(d)));
        } else if (!strcmp(mode, "parse32")) {
            lust_str ls;
            float f = 0;
            int rc;
            ls.ptr = (uint8_t *)line_buf;
            ls.len = (uint64_t)len;
            rc = lust_parse_f32(ls, &f);
            printf("%d %08x\n", rc, rc ? 0u : lust_f32_bits(f));
        } else if (!strcmp(mode, "cast64")) {
            double v = lust_f64_from_bits(strtoull(line_buf, NULL, 16));
            printf("%d\t%d\t%d\t%lld\t%u\t%u\t%u\t%llu\t%08x\n", lust_f64_to_i8(v), lust_f64_to_i16(v), lust_f64_to_i32(v),
                   (long long)lust_f64_to_i64(v), lust_f64_to_u8(v), lust_f64_to_u16(v), lust_f64_to_u32(v),
                   (unsigned long long)lust_f64_to_u64(v), lust_f32_bits(lust_f64_to_f32(v)));
        } else if (!strcmp(mode, "cast32")) {
            float v = lust_f32_from_bits((uint32_t)strtoull(line_buf, NULL, 16));
            printf("%d\t%d\t%d\t%lld\t%u\t%u\t%u\t%llu\t%016llx\n", lust_f32_to_i8(v), lust_f32_to_i16(v), lust_f32_to_i32(v),
                   (long long)lust_f32_to_i64(v), lust_f32_to_u8(v), lust_f32_to_u16(v), lust_f32_to_u32(v),
                   (unsigned long long)lust_f32_to_u64(v), (unsigned long long)lust_f64_bits(lust_f32_to_f64(v)));
        } else if (!strcmp(mode, "cast128") || !strcmp(mode, "cast128f")) {
            uint64_t lo, hi;
            if (mode[7] == 'f') {
                float v = lust_f32_from_bits((uint32_t)strtoull(line_buf, NULL, 16));
                lust_f32_to_i128(v, &lo, &hi);
                print_i128(lo, hi);
                putchar(' ');
                lust_f32_to_u128(v, &lo, &hi);
                print_u128(lo, hi);
            } else {
                double v = lust_f64_from_bits(strtoull(line_buf, NULL, 16));
                lust_f64_to_i128(v, &lo, &hi);
                print_i128(lo, hi);
                putchar(' ');
                lust_f64_to_u128(v, &lo, &hi);
                print_u128(lo, hi);
            }
            putchar('\n');
        } else if (!strcmp(mode, "intcast")) {
            char *sp = strchr(line_buf, ' ');
            uint64_t lo = strtoull(line_buf, NULL, 16), hi = strtoull(sp + 1, NULL, 16);
            printf("%016llx\t%016llx\t%08x\t%08x\t%016llx\t%016llx\t%08x\t%08x\n",
                   (unsigned long long)lust_f64_bits(lust_i64_to_f64((int64_t)lo)),
                   (unsigned long long)lust_f64_bits(lust_u64_to_f64(lo)), lust_f32_bits(lust_i64_to_f32((int64_t)lo)),
                   lust_f32_bits(lust_u64_to_f32(lo)), (unsigned long long)lust_f64_bits(lust_i128_to_f64(lo, hi)),
                   (unsigned long long)lust_f64_bits(lust_u128_to_f64(lo, hi)), lust_f32_bits(lust_i128_to_f32(lo, hi)),
                   lust_f32_bits(lust_u128_to_f32(lo, hi)));
        } else {
            fprintf(stderr, "bad mode %s\n", mode);
            return 2;
        }
    }
}

/* ---------- directed edge assertions (no oracle needed) ---------- */

static int e_pass, e_fail;
static void e_check_str(const char *label, lust_String *s, const char *want) {
    if (s->len == strlen(want) && memcmp(s->ptr, want, s->len) == 0) {
        e_pass++;
    } else {
        e_fail++;
        printf("FAIL %s: got [%.*s] want [%s]\n", label, (int)s->len, (char *)s->ptr, want);
    }
}
static void e_fmt64(double v, lust_float_spec sp, const char *want) {
    lust_String s = lust_string_new();
    lust_fmt_f64(&s, v, sp);
    e_check_str("fmt64", &s, want);
    free(s.ptr);
}
static void e_fmt64b(uint64_t bits, lust_float_spec sp, const char *want) { e_fmt64(lust_f64_from_bits(bits), sp, want); }
static void e_fmt32b(uint32_t bits, lust_float_spec sp, const char *want) {
    lust_String s = lust_string_new();
    lust_fmt_f32(&s, lust_f32_from_bits(bits), sp);
    e_check_str("fmt32", &s, want);
    free(s.ptr);
}
static void e_js64(uint64_t bits, const char *want) {
    lust_String s = lust_string_new();
    lust_fmt_js_f64(&s, lust_f64_from_bits(bits));
    e_check_str("js64", &s, want);
    free(s.ptr);
}
static void e_parse(const char *str, int is64, int want_rc, uint64_t want_bits) {
    lust_str ls;
    int rc;
    uint64_t bits = 0;
    ls.ptr = (uint8_t *)str;
    ls.len = strlen(str);
    if (is64) {
        double d = 0;
        rc = lust_parse_f64(ls, &d);
        bits = lust_f64_bits(d);
    } else {
        float f = 0;
        rc = lust_parse_f32(ls, &f);
        bits = lust_f32_bits(f);
    }
    if (rc == want_rc && (rc != 0 || bits == want_bits))
        e_pass++;
    else {
        e_fail++;
        printf("FAIL parse [%s]: rc=%d bits=%#llx want rc=%d bits=%#llx\n", str, rc, (unsigned long long)bits, want_rc,
               (unsigned long long)want_bits);
    }
}
#define E_CHECK(cond) do { if (cond) e_pass++; else { e_fail++; printf("FAIL check line %d: %s\n", __LINE__, #cond); } } while (0)

static int run_edges(void) {
    static const struct { uint64_t b; const char *d, *g, *e, *js; } rows[] = {
        {0x0000000000000000ull, "0", "0.0", "0e0", "0"},
        {0x8000000000000000ull, "-0", "-0.0", "-0e0", "0"},
        {0x3FF0000000000000ull, "1", "1.0", "1e0", "1"},
        {0xBFF0000000000000ull, "-1", "-1.0", "-1e0", "-1"},
        {0x3FF8000000000000ull, "1.5", "1.5", "1.5e0", "1.5"},
        {0x3FB999999999999Aull, "0.1", "0.1", "1e-1", "0.1"},
        {0x430C6BF526340000ull, "1000000000000000", "1000000000000000.0", "1e15", "1000000000000000"},
        {0x4341C37937E08000ull, "10000000000000000", "1e16", "1e16", "10000000000000000"},
        {0x4415AF1D78B58C40ull, "100000000000000000000", "1e20", "1e20", "100000000000000000000"},
        {0x444B1AE4D6E2EF50ull, "1000000000000000000000", "1e21", "1e21", "1e+21"},
        {0x3F1A36E2EB1C432Dull, "0.0001", "0.0001", "1e-4", "0.0001"},
        {0x3EE4F8B588E368F1ull, "0.00001", "1e-5", "1e-5", "0.00001"},
        {0x3EB0C6F7A0B5ED8Dull, "0.000001", "1e-6", "1e-6", "0.000001"},
        {0x0000000000000001ull, 0, "5e-324", "5e-324", "5e-324"},
        {0x7FEFFFFFFFFFFFFFull, 0, "1.7976931348623157e308", "1.7976931348623157e308", "1.7976931348623157e+308"},
        {0x0010000000000000ull, 0, "2.2250738585072014e-308", "2.2250738585072014e-308", "2.2250738585072014e-308"},
        {0x7FF0000000000000ull, "inf", "inf", "inf", "Infinity"},
        {0xFFF0000000000000ull, "-inf", "-inf", "-inf", "-Infinity"},
        {0x7FF8000000000000ull, "NaN", "NaN", "NaN", "NaN"},
        {0xFFF8000000000000ull, "NaN", "NaN", "NaN", "NaN"},
        /* The tie split: Rust rounds the exact tie up, JS to even. */
        {0x4306501FF5B1980Aull, "785068460487425.3", "785068460487425.3", "7.850684604874253e14", "785068460487425.2"},
        {0x42D11C378BEE3B08ull, "75251554695404.13", "75251554695404.13", "7.525155469540413e13", "75251554695404.12"},
        {0x4330000000000000ull, "4503599627370496", "4503599627370496.0", "4.503599627370496e15", "4503599627370496"},
        {0x4340000000000000ull, "9007199254740992", "9007199254740992.0", "9.007199254740992e15", "9007199254740992"},
    };
    size_t i;
    for (i = 0; i < sizeof rows / sizeof *rows; i++) {
        if (rows[i].d) e_fmt64b(rows[i].b, spec_of(0, NP), rows[i].d);
        e_fmt64b(rows[i].b, spec_of(1, NP), rows[i].g);
        e_fmt64b(rows[i].b, spec_of(2, NP), rows[i].e);
        e_js64(rows[i].b, rows[i].js);
    }
    /* Precision rounding: exact value, ties to even. Indices per oracle.rs. */
    e_fmt64(0.125, spec_table[2], "0.12");
    e_fmt64(0.375, spec_table[2], "0.38");
    e_fmt64(2.5, spec_table[0], "2");
    e_fmt64(3.5, spec_table[0], "4");
    e_fmt64(-2.5, spec_table[0], "-2");
    e_fmt64(-0.5, spec_table[0], "-0");
    e_fmt64(-0.4, spec_table[0], "-0");
    e_fmt64(0.15, spec_table[1], "0.1");
    e_fmt64(0.05, spec_table[1], "0.1");
    e_fmt64(0.25, spec_table[1], "0.2");
    e_fmt64(2.675, spec_table[2], "2.67");
    e_fmt64(9.995, spec_table[2], "9.99");
    e_fmt64(99.995, spec_table[2], "100.00");
    e_fmt64(0.1, spec_table[32], "0.10000000000000000555111512312578270211815834045410");
    e_fmt64(1e20, spec_table[2], "100000000000000000000.00");
    e_fmt64(1e-10, spec_table[7], "0.00");
    e_fmt64(0.00005, spec_table[4], "0.0000500000");
    e_fmt64(9.999, spec_table[10], "1.00e1");
    e_fmt64(9.5, spec_table[9], "1e1");
    e_fmt64(2.5, spec_table[9], "2e0");
    e_fmt64(0.0, spec_table[10], "0.00e0");
    e_fmt64(-0.0, spec_table[2], "-0.00");
    /* Padding, sign, and the zero flag beating fill and align. */
    e_fmt64(1.5, spec_table[13], "         1.5");
    e_fmt64(1.5, spec_table[14], "1.5         ");
    e_fmt64(1.5, spec_table[15], "    1.5     ");
    e_fmt64(1.5, spec_table[17], "0000000001.5");
    e_fmt64(-1.5, spec_table[17], "-000000001.5");
    e_fmt64(1.5, spec_table[18], "+000000001.5");
    e_fmt64(1.5, spec_table[21], "000000001.50");
    e_fmt64(1.5, spec_table[22], "*********1.5");
    e_fmt64(1.5, spec_table[26], "1.5000000000");
    e_fmt64(1.5, spec_table[37], "0000000001.5");
    e_fmt64b(0x7FF0000000000000ull, spec_table[17], "000000000inf");
    e_fmt64b(0x7FF0000000000000ull, spec_table[18], "+00000000inf");
    e_fmt64b(0xFFF0000000000000ull, spec_table[17], "-00000000inf");
    e_fmt64b(0xFFF8000000000000ull, spec_table[17], "000000000NaN");
    e_fmt64b(0x7FF8000000000000ull, spec_table[38], "NaN");
    e_fmt64b(0x7FF8000000000000ull, spec_table[13], "         NaN");
    e_fmt64(1.0, spec_table[27], "1");
    e_fmt64(1.0, spec_table[28], "1.00");
    /* f32 basics. */
    e_fmt32b(0x00000000u, spec_of(0, NP), "0");
    e_fmt32b(0x80000000u, spec_of(1, NP), "-0.0");
    e_fmt32b(0x3F800000u, spec_of(1, NP), "1.0");
    e_fmt32b(0x7F7FFFFFu, spec_of(1, NP), "3.4028235e38");
    e_fmt32b(0x00000001u, spec_of(1, NP), "1e-45");
    e_fmt32b(0x3DCCCCCDu, spec_of(0, NP), "0.1");
    e_fmt32b(0x5A0E1BCAu, spec_of(1, NP), "1e16");
    e_fmt32b(0x7FC00000u, spec_of(0, NP), "NaN");
    /* Parsing: grammar, error kinds, edges. Bits from rustc probes. */
    e_parse("", 1, 1, 0);
    e_parse(" ", 1, 2, 0);
    e_parse(" 1", 1, 2, 0);
    e_parse("1 ", 1, 2, 0);
    e_parse("\t1", 1, 2, 0);
    e_parse("+1", 1, 0, 0x3FF0000000000000ull);
    e_parse("1.", 1, 0, 0x3FF0000000000000ull);
    e_parse(".5", 1, 0, 0x3FE0000000000000ull);
    e_parse(".", 1, 2, 0);
    e_parse("+.", 1, 2, 0);
    e_parse("1e", 1, 2, 0);
    e_parse("1e+", 1, 2, 0);
    e_parse("1E5", 1, 0, 0x40F86A0000000000ull);
    e_parse("1.e5", 1, 0, 0x40F86A0000000000ull);
    e_parse("inf", 1, 0, 0x7FF0000000000000ull);
    e_parse("INFINITY", 1, 0, 0x7FF0000000000000ull);
    e_parse("-inf", 1, 0, 0xFFF0000000000000ull);
    e_parse("NaN", 1, 0, 0x7FF8000000000000ull);
    e_parse("-nan", 1, 0, 0xFFF8000000000000ull);
    e_parse("nAn", 1, 0, 0x7FF8000000000000ull);
    e_parse("infinityy", 1, 2, 0);
    e_parse("1_000", 1, 2, 0);
    e_parse("0x10", 1, 2, 0);
    e_parse("--1", 1, 2, 0);
    e_parse("1e400", 1, 0, 0x7FF0000000000000ull);
    e_parse("-1e400", 1, 0, 0xFFF0000000000000ull);
    e_parse("1e-400", 1, 0, 0);
    e_parse("0e999", 1, 0, 0);
    e_parse("-0", 1, 0, 0x8000000000000000ull);
    e_parse("5e-324", 1, 0, 1);
    e_parse("4e-324", 1, 0, 1);
    e_parse("2.4e-324", 1, 0, 0);
    e_parse("2.5e-324", 1, 0, 1);
    e_parse("2.4703282292062327e-324", 1, 0, 0);
    e_parse("2.4703282292062328e-324", 1, 0, 1);
    e_parse("1.7976931348623157e308", 1, 0, 0x7FEFFFFFFFFFFFFFull);
    e_parse("1.7976931348623158e308", 1, 0, 0x7FEFFFFFFFFFFFFFull);
    e_parse("1.7976931348623159e308", 1, 0, 0x7FF0000000000000ull);
    e_parse("2.2250738585072011e-308", 1, 0, 0x000FFFFFFFFFFFFFull);
    e_parse("2.2250738585072014e-308", 1, 0, 0x0010000000000000ull);
    e_parse("9007199254740993", 1, 0, 0x4340000000000000ull);
    e_parse("0.500000000000000055511151231257827021181583404541015625", 1, 0, 0x3FE0000000000000ull);
    e_parse("1e999999999999999999999", 1, 0, 0x7FF0000000000000ull);
    e_parse("1e-999999999999999999999", 1, 0, 0);
    e_parse("3.4028235e38", 0, 0, 0x7F7FFFFFull);
    e_parse("3.4028236e38", 0, 0, 0x7F800000ull);
    e_parse("1e-45", 0, 0, 1);
    e_parse("7e-46", 0, 0, 0);
    e_parse("0.1", 0, 0, 0x3DCCCCCDull);
    e_parse("16777217", 0, 0, 0x4B800000ull);
    e_parse("nan", 0, 0, 0x7FC00000ull);
    /* Casts. */
    E_CHECK(lust_f64_to_i32(lust_f64_from_bits(0x7FF8000000000000ull)) == 0);
    E_CHECK(lust_f64_to_i32(1e30) == 2147483647);
    E_CHECK(lust_f64_to_i32(-1e30) == -2147483647 - 1);
    E_CHECK(lust_f64_to_i32(1.9) == 1 && lust_f64_to_i32(-1.9) == -1);
    E_CHECK(lust_f64_to_u8(-1.0) == 0 && lust_f64_to_u8(256.0) == 255 && lust_f64_to_u8(255.9) == 255);
    E_CHECK(lust_f64_to_u64(-0.5) == 0);
    E_CHECK(lust_f64_to_i64(9223372036854775807.0) == 9223372036854775807ll);
    E_CHECK(lust_f32_bits(lust_i64_to_f32(16777217)) == 0x4B800000u);
    E_CHECK(lust_f64_bits(lust_i64_to_f64(9007199254740993ll)) == 0x4340000000000000ull);
    E_CHECK(lust_f64_bits(lust_u64_to_f64(~0ull)) == 0x43F0000000000000ull);
    E_CHECK(lust_f32_bits(lust_f64_to_f32(1e40)) == 0x7F800000u);
    E_CHECK(lust_f32_bits(lust_f64_to_f32(0.1)) == 0x3DCCCCCDu);
    E_CHECK(lust_f64_bits(lust_f32_to_f64(lust_f32_from_bits(0x3DCCCCCDu))) == 0x3FB99999A0000000ull);
    {
        uint64_t lo, hi;
        lust_f64_to_u128(1e30, &lo, &hi);
        E_CHECK(lust_f64_bits(lust_u128_to_f64(lo, hi)) == lust_f64_bits(1e30));
        lust_f64_to_i128(-1.5, &lo, &hi);
        E_CHECK(lo == ~0ull && hi == ~0ull);
        lust_f64_to_i128(lust_f64_from_bits(0x7FEFFFFFFFFFFFFFull), &lo, &hi);
        E_CHECK(lo == ~0ull && hi == 0x7FFFFFFFFFFFFFFFull);
    }
    printf("edges: %d passed, %d failed\n", e_pass, e_fail);
    return e_fail ? 1 : 0;
}

/* ---------- benchmark ---------- */

static uint64_t b_st = 0x12345678ABCDEFull;
static uint64_t b_rnd(void) {
    b_st ^= b_st << 13;
    b_st ^= b_st >> 7;
    b_st ^= b_st << 17;
    return b_st;
}
static double b_now(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec + ts.tv_nsec * 1e-9;
}
#define B_N 200000
static int run_bench(void) {
    static uint64_t bits[B_N];
    static char *strs[4096];
    int i;
    double t0, dt;
    uint64_t acc = 0;
    lust_String sink = lust_string_new();
    for (i = 0; i < B_N; i++) {
        uint64_t b = b_rnd();
        if (((b >> 52) & 0x7FF) == 0x7FF) b ^= 0x7FFull << 52;
        bits[i] = b;
    }
    for (i = 0; i < 4096; i++) {
        char tmp[64];
        /* Separate statements: b_rnd() calls in one argument list have
           an unspecified order, so the strings would differ by compiler. */
        unsigned long long a = (unsigned long long)(b_rnd() % 1000000);
        unsigned long long f = (unsigned long long)(b_rnd() % 999983);
        long long e = (long long)((int)(b_rnd() % 600) - 300);
        snprintf(tmp, sizeof tmp, "%llu.%06llue%lld", a, f, e);
        strs[i] = lust_alloc(strlen(tmp) + 1, 1);
        strcpy(strs[i], tmp);
    }
    t0 = b_now();
    for (i = 0; i < B_N; i++) { sink.len = 0; lust_fmt_f64_display(&sink, lust_f64_from_bits(bits[i])); acc += sink.len; }
    dt = b_now() - t0;
    printf("fmt_f64_display  %8.1f ns/call\n", dt * 1e9 / B_N);
    t0 = b_now();
    for (i = 0; i < B_N; i++) { sink.len = 0; lust_fmt_f64_debug(&sink, lust_f64_from_bits(bits[i])); acc += sink.len; }
    dt = b_now() - t0;
    printf("fmt_f64_debug    %8.1f ns/call\n", dt * 1e9 / B_N);
    t0 = b_now();
    for (i = 0; i < B_N; i++) { sink.len = 0; lust_fmt_f64(&sink, lust_f64_from_bits(bits[i]), spec_table[3]); acc += sink.len; }
    dt = b_now() - t0;
    printf("fmt_f64_prec5      %8.1f ns/call\n", dt * 1e9 / B_N);
    t0 = b_now();
    for (i = 0; i < B_N; i++) { sink.len = 0; lust_fmt_f32_display(&sink, lust_f32_from_bits((uint32_t)bits[i])); acc += sink.len; }
    dt = b_now() - t0;
    printf("fmt_f32_display  %8.1f ns/call\n", dt * 1e9 / B_N);
    t0 = b_now();
    for (i = 0; i < B_N; i++) { sink.len = 0; lust_fmt_js_f64(&sink, lust_f64_from_bits(bits[i])); acc += sink.len; }
    dt = b_now() - t0;
    printf("fmt_js_f64       %8.1f ns/call\n", dt * 1e9 / B_N);
    t0 = b_now();
    for (i = 0; i < B_N; i++) {
        lust_str ls;
        double d;
        ls.ptr = (uint8_t *)strs[i & 4095];
        ls.len = strlen(strs[i & 4095]);
        acc += (uint64_t)lust_parse_f64(ls, &d);
        acc += lust_f64_bits(d) & 7;
    }
    dt = b_now() - t0;
    printf("parse_f64        %8.1f ns/call\n", dt * 1e9 / B_N);
    t0 = b_now();
    for (i = 0; i < B_N; i++) {
        lust_str ls;
        float f;
        ls.ptr = (uint8_t *)strs[i & 4095];
        ls.len = strlen(strs[i & 4095]);
        acc += (uint64_t)lust_parse_f32(ls, &f);
        acc += lust_f32_bits(f) & 7;
    }
    dt = b_now() - t0;
    printf("parse_f32        %8.1f ns/call\n", dt * 1e9 / B_N);
    t0 = b_now();
    for (i = 0; i < B_N; i++) acc += (uint64_t)lust_f64_to_i64(lust_f64_from_bits(bits[i])) & 7;
    dt = b_now() - t0;
    printf("f64_to_i64       %8.1f ns/call\n", dt * 1e9 / B_N);
    printf("(sink %llu)\n", (unsigned long long)acc);
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: harness <mode>\n");
        return 2;
    }
    setvbuf(stdout, NULL, _IOFBF, 1 << 20);
    if (!strcmp(argv[1], "edges")) return run_edges();
    if (!strcmp(argv[1], "bench")) return run_bench();
    return mode_stream(argv[1]);
}
