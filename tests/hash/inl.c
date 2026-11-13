/* The hash lookups that run in the caller's unit (hash.h's inline block: lust_hash_find,
   lust_hash_find_or_insert, lust_hash_insert_at) against hash.c's functions. Two tables get
   the same op stream: one through the inline forms, one through the functions (a name in
   parentheses is not a macro call). After each op the results (hit, bucket index) and the
   tables (control bytes, items, growth_left, elements) must be the same. A weak hash mode
   makes long probes, shared tags and tombstones; the tables restart now and then, so small
   tables (fewer buckets than a group, where the insert index can need fix_insert_index)
   come back. Built twice by tests/hash/test.sh: with -DINL_IMPL it is hash.c's unit.
   Usage: inl <seed> <ops>; prints "ok" or the first difference. */
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#ifdef INL_IMPL
void lust_panic_at(const uint8_t *msg, uint64_t len, const char *file, uint32_t line, uint32_t col) {
    (void)msg; (void)len; (void)file; (void)line; (void)col;
    abort();
}
void lust_oom(void) { abort(); }
void lust_oom_size(uint64_t n) { (void)n; abort(); }
#define lust_copy_small(d, s, n) ((void)memmove((d), (s), (size_t)(n)))
#define LUST_IMPL 1
#include "../../src/runtime/hash.h"
#else
#include "../../src/runtime/hash.h"
#include <stdio.h>

#if !defined(lust_hash_find) || !defined(lust_hash_find_or_insert) || !defined(lust_hash_insert_at)
#error "hash.h's inline forms are off in this unit"
#endif

/* An element: the key and a payload word, so elem_size is not a power of two's worth of keys. */
typedef struct { uint64_t key, val, pad; } elem;
static int weak;
static uint64_t hash_key(uint64_t k) {
    if (weak) return ((k % 5) << 57) | (k % 11); /* 5 tags, 11 home buckets */
    k *= 0xf1357aea2e62a9c5ull;
    return k ^ (k >> 29) ^ (k << 41);
}
static int eq_key(void *ctx, const void *e) { return *(const uint64_t *)ctx == ((const elem *)e)->key; }
static uint64_t hash_of(void *ctx, const void *e) { (void)ctx; return hash_key(((const elem *)e)->key); }

static uint64_t rng;
static uint64_t next(void) {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    return rng;
}

static int same(const lust_hash_table *a, const lust_hash_table *b) {
    if (a->bucket_mask != b->bucket_mask || a->items != b->items || a->growth_left != b->growth_left) return 0;
    if (a->items == 0 && a->growth_left == 0) return 1;
    uint64_t n = a->bucket_mask + 1;
    if (memcmp(a->ctrl, b->ctrl, n + LUST_HASH_GROUP_WIDTH)) return 0;
    for (uint64_t i = 0; i < n; i++)
        if (!(a->ctrl[i] & 0x80u) && memcmp(a->data + i * sizeof(elem), b->data + i * sizeof(elem), sizeof(elem))) return 0;
    return 1;
}

int main(int argc, char **argv) {
    if (argc != 3) return 2;
    rng = strtoull(argv[1], 0, 10) * 0x9e3779b97f4a7c15ull | 1;
    long ops = strtol(argv[2], 0, 10);
    lust_hash_table a, b;
    (lust_hash_init)(&a, sizeof(elem), _Alignof(elem));
    (lust_hash_init)(&b, sizeof(elem), _Alignof(elem));
    uint64_t hits = 0, misses = 0, keys = 64;
    for (long op = 0; op < ops; op++) {
        if (op % 5000 == 0) {
            weak = (op / 5000) % 3 == 2;
            keys = 8u << (next() % 8); /* 8 .. 1024 live keys at most */
            (lust_hash_drop_table)(&a, 0, 0);
            (lust_hash_drop_table)(&b, 0, 0);
            (lust_hash_init)(&a, sizeof(elem), _Alignof(elem));
            (lust_hash_init)(&b, sizeof(elem), _Alignof(elem));
        }
        uint64_t r = next(), key = r % keys, h = hash_key(key);
        uint64_t ia = ~0ull, ib = ~0ull;
        switch ((r >> 32) % 4) {
        case 0: case 1: { /* insert or overwrite */
            void *sa = 0, *sb = 0;
            int fa = lust_hash_find_or_insert(&a, h, eq_key, &key, hash_of, 0, &sa, &ia);
            int fb = (lust_hash_find_or_insert)(&b, h, eq_key, &key, hash_of, 0, &sb, &ib);
            if (fa != fb || ia != ib || (fa && (lust_hash_index_of(&a, sa) != ia || lust_hash_index_of(&b, sb) != ib))) {
                printf("FAIL: op %ld find_or_insert key %llu: inline %d@%llu, fn %d@%llu\n", op, (unsigned long long)key, fa,
                       (unsigned long long)ia, fb, (unsigned long long)ib);
                return 1;
            }
            elem e = {key, r, 0};
            if (fa) {
                hits++;
                ((elem *)sa)->val = r;
                ((elem *)sb)->val = r;
            } else {
                misses++;
                *(elem *)lust_hash_insert_at(&a, h, ia) = e;
                *(elem *)(lust_hash_insert_at)(&b, h, ib) = e;
            }
            break;
        }
        case 2: { /* remove: tombstones for the probes to step over */
            int ra = lust_hash_remove(&a, h, eq_key, &key, 0), rb = lust_hash_remove(&b, h, eq_key, &key, 0);
            if (ra != rb) {
                printf("FAIL: op %ld remove key %llu: %d vs %d\n", op, (unsigned long long)key, ra, rb);
                return 1;
            }
            break;
        }
        default: { /* lookup */
            void *sa = lust_hash_find(&a, h, eq_key, &key, &ia), *sb = (lust_hash_find)(&b, h, eq_key, &key, &ib);
            if (!sa != !sb || (sa && ia != ib)) {
                printf("FAIL: op %ld find key %llu: inline %s@%llu, fn %s@%llu\n", op, (unsigned long long)key,
                       sa ? "hit" : "miss", (unsigned long long)ia, sb ? "hit" : "miss", (unsigned long long)ib);
                return 1;
            }
        }
        }
        if (!same(&a, &b)) {
            printf("FAIL: op %ld: the tables differ after key %llu\n", op, (unsigned long long)key);
            return 1;
        }
    }
    printf("ok: %ld ops (%llu hits, %llu inserts), group width %d\n", ops, (unsigned long long)hits,
           (unsigned long long)misses, LUST_HASH_GROUP_WIDTH);
    return 0;
}
#endif
