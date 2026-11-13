/* C harness for the 128-bit runtime helpers in src/runtime/core.c.
   Prints the same lines as oracle.rs (rustc 1.99); test.sh compares
   byte for byte. Random stream: splitmix64, same seed as the oracle. */
#define LUST_IMPL 1
#include "../../src/runtime/runtime.h"
#include "../../src/runtime/core.c"
#include <stdio.h>

/* Rotating buffers: one printf formats several values. */
static char fbufs[8][48];
static int fbuf_at = 0;
static char *fbuf_next(lust_String *s) {
    char *b = fbufs[fbuf_at++ & 7];
    memcpy(b, s->ptr, s->len);
    b[s->len] = 0;
    return b;
}
static const char *fmtu(lust_u128 v) {
    lust_String s = lust_string_new();
    lust_fmt_u128(&s, v);
    const char *r = fbuf_next(&s);
    lust_string_drop(&s);
    return r;
}
static const char *fmti(lust_i128 v) {
    lust_String s = lust_string_new();
    lust_fmt_i128(&s, v);
    const char *r = fbuf_next(&s);
    lust_string_drop(&s);
    return r;
}

static lust_u128 U(uint64_t lo, uint64_t hi) {
    lust_u128 r;
    r.lo = lo;
    r.hi = hi;
    return r;
}

static void show(lust_u128 a, lust_u128 b) {
    lust_i128 ia = lust_i128_from_u128(a), ib = lust_i128_from_u128(b);
    printf("a=%s b=%s\n", fmtu(a), fmtu(b));
    printf("uadd=%s usub=%s umul=%s\n", fmtu(lust_u128_add(a, b)), fmtu(lust_u128_sub(a, b)), fmtu(lust_u128_mul(a, b)));
    printf("and=%s or=%s xor=%s not=%s\n", fmtu(lust_u128_and(a, b)), fmtu(lust_u128_or(a, b)), fmtu(lust_u128_xor(a, b)), fmtu(lust_u128_not(a)));
    if (b.lo || b.hi) {
        lust_u128 r;
        lust_u128 q = lust_u128_divmod(a, b, &r);
        printf("udiv=%s urem=%s\n", fmtu(q), fmtu(r));
    }
    int is_min = ia.lo == 0 && ia.hi == 0x8000000000000000ull;
    int is_neg1 = ib.lo == 0xFFFFFFFFFFFFFFFFull && ib.hi == 0xFFFFFFFFFFFFFFFFull;
    if ((ib.lo || ib.hi) && !(is_min && is_neg1)) {
        printf("idiv=%s irem=%s\n", fmti(lust_div_i128(ia, ib, 0)), fmti(lust_rem_i128(ia, ib, 0)));
    }
    uint64_t kk;
    (void)lust_u128_divmod64(b, 128, &kk);
    uint32_t k = (uint32_t)kk;
    printf("shl=%s ushr=%s ishr=%s\n", fmtu(lust_u128_shl(a, k)), fmtu(lust_u128_shr(a, k)), fmti(lust_i128_shr(ia, k)));
    printf("ucmp=%d icmp=%d\n", (int)lust_u128_cmp(a, b), (int)lust_i128_cmp(ia, ib));
    lust_i128 z = {0, 0};
    printf("fmtu=%s fmti=%s neg=%s\n", fmtu(a), fmti(ia), fmti(lust_i128_sub(z, ia)));
}

static uint64_t state = 0x123456789ABCDEF0ull;
static uint64_t next(void) {
    state += 0x9E3779B97F4A7C15ull;
    uint64_t z = state;
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ull;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBull;
    return z ^ (z >> 31);
}

static uint64_t arg_u64(const char *s) { return strtoull(s, 0, 10); }

int main(int argc, char **argv) {
    static const uint32_t one_line[1] = {1};
    lust_file = "int128-harness";
    lust_fn_line = one_line;
    if (argc > 1 && strcmp(argv[1], "checked") == 0) {
        lust_u128 a = U(arg_u64(argv[3]), arg_u64(argv[4]));
        lust_u128 b = U(arg_u64(argv[5]), arg_u64(argv[6]));
        lust_i128 ia = lust_i128_from_u128(a), ib = lust_i128_from_u128(b);
        const char *op = argv[2];
        if (!strcmp(op, "uadd")) printf("%s\n", fmtu(lust_add_u128(a, b, 0)));
        else if (!strcmp(op, "usub")) printf("%s\n", fmtu(lust_sub_u128(a, b, 0)));
        else if (!strcmp(op, "umul")) printf("%s\n", fmtu(lust_mul_u128(a, b, 0)));
        else if (!strcmp(op, "udiv")) printf("%s\n", fmtu(lust_div_u128(a, b, 0)));
        else if (!strcmp(op, "urem")) printf("%s\n", fmtu(lust_rem_u128(a, b, 0)));
        else if (!strcmp(op, "iadd")) printf("%s\n", fmti(lust_add_i128(ia, ib, 0)));
        else if (!strcmp(op, "isub")) printf("%s\n", fmti(lust_sub_i128(ia, ib, 0)));
        else if (!strcmp(op, "imul")) printf("%s\n", fmti(lust_mul_i128(ia, ib, 0)));
        else if (!strcmp(op, "idiv")) printf("%s\n", fmti(lust_div_i128(ia, ib, 0)));
        else if (!strcmp(op, "irem")) printf("%s\n", fmti(lust_rem_i128(ia, ib, 0)));
        else if (!strcmp(op, "ineg")) printf("%s\n", fmti(lust_neg_i128(ia, 0)));
        else if (!strcmp(op, "iabs")) printf("%s\n", fmti(lust_abs_i128(ia, 0)));
        else if (!strcmp(op, "upow")) printf("%s\n", fmtu(lust_pow_u128(a, (uint32_t)arg_u64(argv[5]), 0)));
        else if (!strcmp(op, "ipow")) printf("%s\n", fmti(lust_pow_i128(ia, (uint32_t)arg_u64(argv[5]), 0)));
        return 0;
    }
    printf("size=%d align=%d\n", (int)sizeof(lust_u128), (int)_Alignof(lust_u128));
    lust_u128 e[18];
    int en = 0;
    e[en++] = U(0, 0); e[en++] = U(1, 0); e[en++] = U(2, 0); e[en++] = U(3, 0); e[en++] = U(10, 0);
    e[en++] = U(0xFFFFFFFFull, 0); e[en++] = U(0x100000000ull, 0); e[en++] = U(0xFFFFFFFFFFFFFFFFull, 0);
    e[en++] = U(0, 1); e[en++] = U(0xFFFFFFFFFFFFFFFFull, 0xFFFFFFFFFFFFFFFFull);
    e[en++] = U(0xFFFFFFFFFFFFFFFEull, 0xFFFFFFFFFFFFFFFFull);
    e[en++] = U(0, 0x8000000000000000ull); e[en++] = U(0xFFFFFFFFFFFFFFFFull, 0x7FFFFFFFFFFFFFFFull);
    /* 10^19 and 10^38 through checked pow-free construction: 10^19 fits
       one word; 10^38 = 10^19 * 10^19. */
    e[en++] = U(10000000000000000000ull, 0);
    e[en++] = lust_u128_mul(U(10000000000000000000ull, 0), U(10000000000000000000ull, 0));
    e[en++] = U(0, 0x8000000000000000ull);
    e[en++] = U(0x9ABCDEF00BADF00Dull, 0xDEADBEEF12345678ull);
    e[en++] = U(0xFFFFFFFFFFFFFFFFull, 0xFFFFFFFFFFFFFFFFull);
    for (int i = 0; i < en; i++)
        for (int j = 0; j < en; j++) show(e[i], e[j]);
    for (int i = 0; i < 4000; i++) {
        /* Match the oracle stream order: hi first, then lo. */
        lust_u128 a = U(0, 0), b = U(0, 0);
        a.hi = next();
        a.lo = next();
        b.hi = next();
        b.lo = next();
        show(a, b);
        show(a, U(b.lo & 0xFFFF, 0));
        show(a, U(b.lo, 0));
    }
    return 0;
}
