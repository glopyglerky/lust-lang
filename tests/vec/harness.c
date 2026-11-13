/* C harness for the erased Vec core and the alloc family in
   src/runtime/core.c. Prints the same capacity lines as oracle.rs
   (rustc 1.99) for the same operation sequences; test.sh compares
   byte for byte. The `alloc` mode checks aligned allocation directly. */
#define LUST_IMPL 1
#include "../../src/runtime/runtime.h"
#include "../../src/runtime/core.c"
#include <stdio.h>

static lust_Vec new_vec(void) {
    lust_Vec v = {0, 0, 0};
    return v;
}

static void drop_vec(lust_Vec *v, uint64_t elem, uint64_t align) {
    lust_dealloc(v->ptr, v->cap == 18446744073709551615ull ? 0 : v->cap * elem, align);
}

static void seq(const char *name, uint64_t elem, uint64_t align) {
    static const uint64_t ns[6] = {1, 2, 3, 5, 10, 100};
    lust_Vec v = new_vec();
    printf("%s push:", name);
    for (int i = 0; i < 20; i++) {
        lust_vec_push_slot(&v, elem, align);
        printf(" %llu", (unsigned long long)v.cap);
    }
    printf("\n");
    drop_vec(&v, elem, align);

    v = new_vec();
    printf("%s reserve:", name);
    for (int i = 0; i < 6; i++) {
        lust_vec_reserve(&v, ns[i], elem, align);
        printf(" r%llu=%llu", (unsigned long long)ns[i], (unsigned long long)v.cap);
    }
    printf("\n");
    drop_vec(&v, elem, align);

    v = new_vec();
    printf("%s reserve_exact:", name);
    for (int i = 0; i < 6; i++) {
        lust_vec_reserve_exact(&v, ns[i], elem, align);
        printf(" r%llu=%llu", (unsigned long long)ns[i], (unsigned long long)v.cap);
    }
    printf("\n");
    drop_vec(&v, elem, align);

    v = new_vec();
    for (int i = 0; i < 10; i++) lust_vec_push_slot(&v, elem, align);
    printf("%s shrink: start=%llu", name, (unsigned long long)v.cap);
    lust_vec_shrink_to(&v, 12, elem, align);
    printf(" to12=%llu", (unsigned long long)v.cap);
    lust_vec_shrink_to(&v, 10, elem, align);
    printf(" to10=%llu", (unsigned long long)v.cap);
    v.len = 3; /* truncate(3) */
    lust_vec_shrink_to(&v, 5, elem, align);
    printf(" len3to5=%llu", (unsigned long long)v.cap);
    lust_vec_shrink_to(&v, 0, elem, align); /* shrink_to_fit clamps at len */
    printf(" fit=%llu", (unsigned long long)v.cap);
    v.len = 0; /* clear */
    lust_vec_shrink_to(&v, 0, elem, align);
    printf(" emptyfit=%llu\n", (unsigned long long)v.cap);
    drop_vec(&v, elem, align);
}

static int failures = 0;
#define CHECK(cond, msg) do { if (!(cond)) { fprintf(stderr, "FAIL %s:%d: %s\n", __FILE__, __LINE__, msg); failures++; } } while (0)

static int alloc_checks(void) {
    static const uint64_t aligns[8] = {1, 2, 8, 16, 64, 4096, 65536, 4194304};
    static const uint64_t sizes[4] = {1, 13, 100, 4194304};
    for (int a = 0; a < 8; a++) {
        for (int s = 0; s < 4; s++) {
            uint64_t size = sizes[s], align = aligns[a];
            uint8_t *p = lust_alloc(size, align);
            CHECK(((uintptr_t)p & (align - 1)) == 0, "alloc alignment");
            memset(p, 0xAB, size);
            uint8_t *q = lust_realloc(p, size, align, size + 7);
            CHECK(((uintptr_t)q & (align - 1)) == 0, "realloc alignment");
            CHECK(q[0] == 0xAB && q[size - 1] == 0xAB, "realloc keeps bytes");
            lust_dealloc(q, size + 7, align);
        }
    }
    uint8_t *z = lust_alloc_zeroed(64, 64);
    for (int i = 0; i < 64; i++) CHECK(z[i] == 0, "alloc_zeroed zeroes");
    lust_dealloc(z, 64, 64);
    void *d = lust_alloc(0, 4096);
    CHECK(((uintptr_t)d & 4095) == 0, "zero-size dangling is aligned");
    lust_dealloc(d, 0, 4096);
    if (!failures) printf("alloc ok\n");
    return failures != 0;
}

int main(int argc, char **argv) {
    if (argc > 1 && strcmp(argv[1], "alloc") == 0) return alloc_checks();
    seq("u8", 1, 1);
    seq("u16", 2, 2);
    seq("u32", 4, 4);
    seq("u64", 8, 8);
    seq("u128", 16, 16);
    seq("b1024", 1024, 1);
    seq("b1025", 1025, 1);
    seq("zst", 0, 1);

    /* String is Vec<u8>: same sequences through the string core. */
    lust_String s = lust_string_new();
    printf("String push_str:");
    static const uint64_t adds[6] = {1, 7, 1, 8, 100, 1000};
    char buf[1000];
    memset(buf, 'x', sizeof buf);
    for (int i = 0; i < 6; i++) {
        lust_string_push_bytes(&s, buf, adds[i]);
        printf(" +%llu=%llu", (unsigned long long)adds[i], (unsigned long long)s.cap);
    }
    printf("\n");
    lust_string_drop(&s);
    s = lust_string_new();
    lust_string_reserve(&s, 3);
    printf("String reserve3=%llu", (unsigned long long)s.cap);
    lust_string_reserve_exact(&s, 3);
    printf(" reserve_exact3=%llu\n", (unsigned long long)s.cap);
    lust_string_drop(&s);

    lust_Vec a = lust_vec_with_capacity(0, 1, 1);
    lust_Vec b = lust_vec_with_capacity(3, 1, 1);
    lust_Vec c = lust_vec_with_capacity(100, 1, 1);
    printf("with_cap u8: %llu %llu %llu\n", (unsigned long long)a.cap, (unsigned long long)b.cap, (unsigned long long)c.cap);
    drop_vec(&a, 1, 1);
    drop_vec(&b, 1, 1);
    drop_vec(&c, 1, 1);

    /* collect from a trusted-length source reserves exact, then the
       length is the capacity. Modeled as reserve_exact + pushes. */
    lust_Vec t = new_vec();
    lust_vec_reserve_exact(&t, 34, 1, 1);
    for (int i = 0; i < 34; i++) lust_vec_push_slot(&t, 1, 1);
    printf("collect34 u8 cap=%llu len=%llu\n", (unsigned long long)t.cap, (unsigned long long)t.len);
    drop_vec(&t, 1, 1);
    /* collect from size_hint (0, None): plain pushes. */
    t = new_vec();
    for (int i = 0; i < 34; i++) lust_vec_push_slot(&t, 1, 1);
    printf("filter collect cap=%llu len=%llu\n", (unsigned long long)t.cap, (unsigned long long)t.len);
    drop_vec(&t, 1, 1);
    return 0;
}
