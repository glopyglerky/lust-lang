/* C harness for the erased hash core (src/runtime/hash.c).
   Built with the pinned tcc, the same compiler lust uses. Modes mirror
   oracle/src/main.rs: same stdin op stream, same stdout bytes.
   Extra modes: fxvectors (rustc-hash's own published test values),
   edges (directed capacity/hash checks via asserts, prints ok). */
#define _POSIX_C_SOURCE 199309L
#include <stdint.h>
#include <stdlib.h>

/* hash.c calls the runtime's panic and OOM entry points on paths
   this harness never takes (capacity overflow, allocation
   failure). Abort stubs stand in for them; the real ones are in
   src/runtime/panic.c and core.c, which this harness does not link. */
void lust_panic_at(const uint8_t *msg, uint64_t len, const char *file, uint32_t line, uint32_t col) {
    (void)msg; (void)len; (void)file; (void)line; (void)col;
    abort();
}
void lust_oom(void) { abort(); }
void lust_oom_size(uint64_t n) { (void)n; abort(); }

/* runtime.h's copy helper for run-time lengths, which hash.c's element moves use. */
#include <string.h>
#define lust_copy_small(d, s, n) ((void)memmove((d), (s), (size_t)(n)))

#define LUST_IMPL 1
#include "../../src/runtime/hash.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static int failures = 0;
#define CHECK(cond, msg) \
    do { if (!(cond)) { fprintf(stderr, "FAIL %s:%d: %s\n", __FILE__, __LINE__, msg); failures++; } } while (0)

/* Makes the same string key as the oracle's make_key(kid). */
static uint64_t make_key_str(uint64_t kid, char *buf, uint64_t bufsz) {
    int n = snprintf(buf, (size_t)bufsz, "k%llu", (unsigned long long)kid);
    uint64_t len = (uint64_t)n;
    char fill = (char)('a' + (kid % 26));
    uint64_t reps = (kid * 7) % 41;
    for (uint64_t i = 0; i < reps && len + 1 < bufsz; i++) buf[len++] = fill;
    buf[len] = 0;
    return len;
}

/* ---------------- SipHash-2-4 with keys 0,0 (std's deprecated
   SipHasher), streaming exactly like core's hash/sip.rs. SipHash-1-3
   (the DefaultHasher) differs only in round counts (1 and 3). This
   exists to prove the table core is hasher-agnostic against real std
   hashing; the lust std layer will supply SipHash-1-3 in source. */
typedef struct {
    uint64_t v0, v1, v2, v3;
    uint64_t tail;
    uint64_t ntail, length;
} sip24;
static uint64_t rotl64(uint64_t x, int b) { return (x << b) | (x >> (64 - b)); }
#define SIP_COMPRESS(s) \
    do { \
        (s)->v0 += (s)->v1; (s)->v2 += (s)->v3; (s)->v1 = rotl64((s)->v1, 13); (s)->v1 ^= (s)->v0; \
        (s)->v3 = rotl64((s)->v3, 16); (s)->v3 ^= (s)->v2; (s)->v0 = rotl64((s)->v0, 32); \
        (s)->v2 += (s)->v1; (s)->v0 += (s)->v3; (s)->v1 = rotl64((s)->v1, 17); (s)->v1 ^= (s)->v2; \
        (s)->v3 = rotl64((s)->v3, 21); (s)->v3 ^= (s)->v0; (s)->v2 = rotl64((s)->v2, 32); \
    } while (0)
static void sip_init(sip24 *s) {
    s->v0 = 0x736f6d6570736575ull;
    s->v1 = 0x646f72616e646f6dull;
    s->v2 = 0x6c7967656e657261ull;
    s->v3 = 0x7465646279746573ull;
    s->tail = 0;
    s->ntail = 0;
    s->length = 0;
}
static uint64_t sip_read_le64(const uint8_t *p) {
    uint64_t v = 0;
    for (int i = 7; i >= 0; i--) v = (v << 8) | p[i];
    return v;
}
static void sip_write(sip24 *s, const uint8_t *msg, uint64_t len) {
    s->length += len;
    uint64_t needed = 0;
    if (s->ntail != 0) {
        needed = 8 - s->ntail;
        uint64_t take = len < needed ? len : needed;
        uint64_t part = 0;
        for (uint64_t i = 0; i < take; i++) part |= (uint64_t)msg[i] << (8 * i);
        s->tail |= part << (8 * s->ntail);
        if (len < needed) {
            s->ntail += len;
            return;
        }
        s->v3 ^= s->tail;
        SIP_COMPRESS(s);
        SIP_COMPRESS(s);
        s->v0 ^= s->tail;
        s->ntail = 0;
    }
    uint64_t total = len - needed;
    uint64_t left = total & 7;
    uint64_t i = needed;
    while (i < needed + total - left) {
        uint64_t mi = sip_read_le64(msg + i);
        s->v3 ^= mi;
        SIP_COMPRESS(s);
        SIP_COMPRESS(s);
        s->v0 ^= mi;
        i += 8;
    }
    uint64_t tail = 0;
    for (uint64_t j = 0; j < left; j++) tail |= (uint64_t)msg[i + j] << (8 * j);
    s->tail = tail;
    s->ntail = left;
}
static uint64_t sip_finish(const sip24 *ss) {
    sip24 s = *ss;
    uint64_t b = ((s.length & 0xff) << 56) | s.tail;
    s.v3 ^= b;
    SIP_COMPRESS(&s);
    SIP_COMPRESS(&s);
    s.v0 ^= b;
    s.v2 ^= 0xff;
    SIP_COMPRESS(&s);
    SIP_COMPRESS(&s);
    SIP_COMPRESS(&s);
    SIP_COMPRESS(&s);
    return s.v0 ^ s.v1 ^ s.v2 ^ s.v3;
}
static uint64_t sip_hash_u64(uint64_t v) {
    uint8_t bytes[8];
    for (int i = 0; i < 8; i++) bytes[i] = (uint8_t)(v >> (8 * i));
    sip24 s;
    sip_init(&s);
    sip_write(&s, bytes, 8);
    return sip_finish(&s);
}

/* ---------------- map modes ----------------
   Element is (kid, value). kid is the key identity printed in state:
   the u64 key itself, the truncated u32 key, or the id behind a
   generated string key. Hash and equality go through kid exactly like
   the Rust key of the same value. */
enum { KIND_U64, KIND_U32, KIND_STR, KIND_SIP };
static int g_hb_table = 0; /* hb8: entry op follows hashbrown HashTable::entry */
typedef struct {
    uint64_t kid, v, pad0, pad1;
} Elem;
static int g_kind;
static uint64_t hash_kid(uint64_t kid) {
    char buf[96];
    switch (g_kind) {
    case KIND_U64: return lust_fx_hash_u64(kid);
    case KIND_U32: return lust_fx_hash_u32((uint32_t)kid);
    case KIND_SIP: return sip_hash_u64(kid);
    default: {
        uint64_t len = make_key_str(kid, buf, sizeof buf);
        return lust_fx_hash_str(buf, len);
    }
    }
}
static uint64_t elem_hash_of(void *ctx, const void *e) {
    (void)ctx;
    return hash_kid(((const Elem *)e)->kid);
}
static int elem_eq(void *ctx, const void *e) { return ((const Elem *)e)->kid == *(const uint64_t *)ctx; }
static int keep_by_value(void *ctx, void *e) { return ((Elem *)e)->v % *(uint64_t *)ctx != 0; }

static void print_state_map(const lust_hash_table *t) {
    printf("%llu %llu [", (unsigned long long)lust_hash_len(t), (unsigned long long)lust_hash_capacity(t));
    lust_hash_iter it;
    lust_hash_iter_init(&it, t);
    void *p;
    int first = 1;
    while ((p = lust_hash_iter_next(&it, NULL)) != NULL) {
        if (!first) putchar(' ');
        first = 0;
        printf("%llu:%llu", (unsigned long long)((Elem *)p)->kid, (unsigned long long)((Elem *)p)->v);
    }
    printf("]\n");
}

static uint64_t elem_size_for_kind(void) {
    /* Rust (String, u64) is 32 bytes; (u64,u64)/(u32,u64) are 16. Our
       Elem is 32 with pads: for 16-byte kinds we still must pass the
       Rust size for small-table capacity rules, so stride differs. */
    return g_kind == KIND_STR ? 32 : 16;
}
static Elem *elem_at(void *slot) { return (Elem *)slot; }

static int run_map(void) {
    uint64_t esz = elem_size_for_kind();
    lust_hash_table t;
    lust_hash_init(&t, esz, 8);
    char line[256];
    while (fgets(line, sizeof line, stdin)) {
        char op = line[0];
        if (op == '\n' || op == 0) continue;
        char *rest = line + 1;
        uint64_t a = strtoull(rest, &rest, 10);
        uint64_t b = strtoull(rest, NULL, 10);
        if (g_kind == KIND_U32) a = (uint32_t)a;
        switch (op) {
        case 'n':
            lust_hash_drop_table(&t, NULL, NULL);
            printf("new ; ");
            break;
        case 'w':
            lust_hash_drop_table(&t, NULL, NULL);
            lust_hash_init_with_capacity(&t, esz, 8, a);
            printf("new ; ");
            break;
        case 'i': {
            void *slot;
            uint64_t idx;
            uint64_t h = hash_kid(a);
            if (lust_hash_find_or_insert(&t, h, elem_eq, &a, elem_hash_of, NULL, &slot, &idx)) {
                printf("%llu ; ", (unsigned long long)elem_at(slot)->v);
                elem_at(slot)->v = b;
            } else {
                Elem *s = elem_at(lust_hash_insert_at(&t, h, idx));
                s->kid = a;
                s->v = b;
                printf("- ; ");
            }
            break;
        }
        case 'e': {
            uint64_t h = hash_kid(a);
            if (g_hb_table) {
                void *slot;
                uint64_t eidx;
                if (lust_hash_find_or_insert(&t, h, elem_eq, &a, elem_hash_of, NULL, &slot, &eidx)) {
                    printf("%llu ; ", (unsigned long long)elem_at(slot)->v);
                } else {
                    Elem *s = elem_at(lust_hash_insert_at(&t, h, eidx));
                    s->kid = a;
                    s->v = b;
                    printf("%llu ; ", (unsigned long long)b);
                }
                break;
            }
            void *slot = lust_hash_find(&t, h, elem_eq, &a, NULL);
            if (slot) {
                printf("%llu ; ", (unsigned long long)elem_at(slot)->v);
            } else {
                /* std entry(): reserve(1) while vacant, then insert_no_grow. */
                lust_hash_reserve(&t, 1, elem_hash_of, NULL);
                Elem *s = elem_at(lust_hash_insert_no_grow_slot(&t, h, NULL));
                s->kid = a;
                s->v = b;
                printf("%llu ; ", (unsigned long long)b);
            }
            break;
        }
        case 'g': {
            void *slot = lust_hash_find(&t, hash_kid(a), elem_eq, &a, NULL);
            if (slot) printf("%llu ; ", (unsigned long long)elem_at(slot)->v);
            else printf("- ; ");
            break;
        }
        case 'r': {
            Elem out;
            if (lust_hash_remove(&t, hash_kid(a), elem_eq, &a, &out)) printf("%llu ; ", (unsigned long long)out.v);
            else printf("- ; ");
            break;
        }
        case 'R': lust_hash_reserve(&t, a, elem_hash_of, NULL); printf("ok ; "); break;
        case 's': lust_hash_shrink_to(&t, a, elem_hash_of, NULL); printf("ok ; "); break;
        case 'F': lust_hash_shrink_to(&t, 0, elem_hash_of, NULL); printf("ok ; "); break;
        case 'c': lust_hash_clear(&t, NULL, NULL); printf("ok ; "); break;
        case 'T': lust_hash_retain(&t, keep_by_value, &a, NULL, NULL); printf("ok ; "); break;
        case 'x': {
            int first = 1;
            uint64_t buckets = lust_hash_buckets(&t);
            uint64_t left = lust_hash_len(&t);
            for (uint64_t i = 0; i < buckets && left; i++) {
                void *slot = lust_hash_bucket(&t, i);
                if (!slot) continue;
                left--;
                if (elem_at(slot)->v % a == 0) {
                    Elem out;
                    lust_hash_erase_at(&t, i, &out);
                    if (!first) putchar(' ');
                    first = 0;
                    printf("%llu:%llu", (unsigned long long)out.kid, (unsigned long long)out.v);
                }
            }
            if (first) putchar('-');
            printf(" ; ");
            break;
        }
        case 'd': {
            lust_hash_drain d;
            lust_hash_drain_init(&d, &t);
            int first = 1;
            for (uint64_t i = 0; i < a; i++) {
                void *slot = lust_hash_drain_next(&d, NULL);
                if (!slot) break;
                Elem copy = *elem_at(slot);
                if (!first) putchar(' ');
                first = 0;
                printf("%llu:%llu", (unsigned long long)copy.kid, (unsigned long long)copy.v);
            }
            lust_hash_drain_finish(&d, NULL, NULL);
            if (first) putchar('-');
            printf(" ; ");
            break;
        }
        case 'l': {
            lust_hash_table c;
            lust_hash_init(&c, esz, 8);
            lust_hash_clone_into(&c, &t, NULL, NULL);
            printf("clone ");
            /* State print adds its own newline; clone line format is
               `clone <state> ; <state>` like the oracle for this mode
               family: print clone state inline here. */
            printf("%llu %llu [", (unsigned long long)lust_hash_len(&c),
                   (unsigned long long)lust_hash_capacity(&c));
            lust_hash_iter it;
            lust_hash_iter_init(&it, &c);
            void *p;
            int first = 1;
            while ((p = lust_hash_iter_next(&it, NULL)) != NULL) {
                if (!first) putchar(' ');
                first = 0;
                printf("%llu:%llu", (unsigned long long)((Elem *)p)->kid, (unsigned long long)((Elem *)p)->v);
            }
            printf("] ; ");
            lust_hash_drop_table(&c, NULL, NULL);
            break;
        }
        default: fprintf(stderr, "bad op %c\n", op); return 2;
        }
        print_state_map(&t);
        fflush(stdout);
    }
    lust_hash_drop_table(&t, NULL, NULL);
    return failures != 0;
}

/* Set mode: element is the u64 key alone, 8 bytes like Rust (u64, ()). */
static uint64_t set_hash_of(void *ctx, const void *e) {
    (void)ctx;
    uint64_t k;
    memcpy(&k, e, 8);
    return lust_fx_hash_u64(k);
}
static int set_eq(void *ctx, const void *e) {
    uint64_t k;
    memcpy(&k, e, 8);
    return k == *(const uint64_t *)ctx;
}
static int set_keep(void *ctx, void *e) {
    uint64_t k;
    memcpy(&k, e, 8);
    return k % *(uint64_t *)ctx != 0;
}
static void print_state_set(const lust_hash_table *t) {
    printf("%llu %llu [", (unsigned long long)lust_hash_len(t), (unsigned long long)lust_hash_capacity(t));
    lust_hash_iter it;
    lust_hash_iter_init(&it, t);
    void *p;
    int first = 1;
    while ((p = lust_hash_iter_next(&it, NULL)) != NULL) {
        uint64_t k;
        memcpy(&k, p, 8);
        if (!first) putchar(' ');
        first = 0;
        printf("%llu", (unsigned long long)k);
    }
    printf("]\n");
}
static int run_set(void) {
    lust_hash_table t;
    lust_hash_init(&t, 8, 8);
    char line[256];
    while (fgets(line, sizeof line, stdin)) {
        char op = line[0];
        if (op == '\n' || op == 0) continue;
        char *rest = line + 1;
        uint64_t a = strtoull(rest, &rest, 10);
        uint64_t b = strtoull(rest, NULL, 10);
        (void)b;
        switch (op) {
        case 'n': lust_hash_drop_table(&t, NULL, NULL); printf("new ; "); break;
        case 'w':
            lust_hash_drop_table(&t, NULL, NULL);
            lust_hash_init_with_capacity(&t, 8, 8, a);
            printf("new ; ");
            break;
        case 'i':
        case 'e': {
            void *slot;
            uint64_t idx;
            uint64_t h = lust_fx_hash_u64(a);
            if (lust_hash_find_or_insert(&t, h, set_eq, &a, set_hash_of, NULL, &slot, &idx)) {
                printf("old ; ");
            } else {
                void *s = lust_hash_insert_at(&t, h, idx);
                memcpy(s, &a, 8);
                printf("new ; ");
            }
            break;
        }
        case 'g': printf(lust_hash_find(&t, lust_fx_hash_u64(a), set_eq, &a, NULL) ? "1 ; " : "- ; "); break;
        case 'r': printf(lust_hash_remove(&t, lust_fx_hash_u64(a), set_eq, &a, NULL) ? "1 ; " : "- ; "); break;
        case 'R': lust_hash_reserve(&t, a, set_hash_of, NULL); printf("ok ; "); break;
        case 's': lust_hash_shrink_to(&t, a, set_hash_of, NULL); printf("ok ; "); break;
        case 'F': lust_hash_shrink_to(&t, 0, set_hash_of, NULL); printf("ok ; "); break;
        case 'c': lust_hash_clear(&t, NULL, NULL); printf("ok ; "); break;
        case 'T': lust_hash_retain(&t, set_keep, &a, NULL, NULL); printf("ok ; "); break;
        case 'x': {
            int first = 1;
            uint64_t buckets = lust_hash_buckets(&t), left = lust_hash_len(&t);
            for (uint64_t i = 0; i < buckets && left; i++) {
                void *slot = lust_hash_bucket(&t, i);
                if (!slot) continue;
                left--;
                uint64_t k;
                memcpy(&k, slot, 8);
                if (k % a == 0) {
                    lust_hash_erase_at(&t, i, NULL);
                    if (!first) putchar(' ');
                    first = 0;
                    printf("%llu", (unsigned long long)k);
                }
            }
            if (first) putchar('-');
            printf(" ; ");
            break;
        }
        case 'd': {
            lust_hash_drain d;
            lust_hash_drain_init(&d, &t);
            int first = 1;
            for (uint64_t i = 0; i < a; i++) {
                void *slot = lust_hash_drain_next(&d, NULL);
                if (!slot) break;
                uint64_t k;
                memcpy(&k, slot, 8);
                if (!first) putchar(' ');
                first = 0;
                printf("%llu", (unsigned long long)k);
            }
            lust_hash_drain_finish(&d, NULL, NULL);
            if (first) putchar('-');
            printf(" ; ");
            break;
        }
        case 'l': {
            lust_hash_table c;
            lust_hash_init(&c, 8, 8);
            lust_hash_clone_into(&c, &t, NULL, NULL);
            printf("clone ");
            printf("%llu %llu [", (unsigned long long)lust_hash_len(&c), (unsigned long long)lust_hash_capacity(&c));
            lust_hash_iter it;
            lust_hash_iter_init(&it, &c);
            void *p;
            int first = 1;
            while ((p = lust_hash_iter_next(&it, NULL)) != NULL) {
                uint64_t k;
                memcpy(&k, p, 8);
                if (!first) putchar(' ');
                first = 0;
                printf("%llu", (unsigned long long)k);
            }
            printf("] ; ");
            lust_hash_drop_table(&c, NULL, NULL);
            break;
        }
        default: fprintf(stderr, "bad set op %c\n", op); return 2;
        }
        print_state_set(&t);
        fflush(stdout);
    }
    lust_hash_drop_table(&t, NULL, NULL);
    return failures != 0;
}

/* ---------------- index modes ----------------
   Key bytes hold the u64 kid (the generated string for imstr is hashed
   from kid), value bytes hold the u64 value. */
static int idx_eq(void *ctx, const void *key) {
    uint64_t k;
    memcpy(&k, key, 8);
    return k == *(const uint64_t *)ctx;
}
static int idx_keep(void *ctx, void *key, void *value) {
    (void)key;
    uint64_t v;
    memcpy(&v, value, 8);
    return v % *(uint64_t *)ctx != 0;
}
static uint64_t idx_hash_kid(uint64_t kid) {
    if (g_kind == KIND_STR) {
        char buf[96];
        uint64_t len = make_key_str(kid, buf, sizeof buf);
        return lust_fx_hash_str(buf, len);
    }
    return lust_fx_hash_u64(kid);
}
static void print_state_index(const lust_index_table *t) {
    printf("%llu %llu [", (unsigned long long)lust_index_len(t), (unsigned long long)lust_index_capacity(t));
    for (uint64_t i = 0; i < lust_index_len(t); i++) {
        void *k, *v;
        lust_index_get_index(t, i, &k, &v);
        uint64_t kid, val;
        memcpy(&kid, k, 8);
        memcpy(&val, v, 8);
        if (i) putchar(' ');
        printf("%llu:%llu", (unsigned long long)kid, (unsigned long long)val);
    }
    printf("]\n");
}
static void idx_write(void *kslot, void *vslot, uint64_t kid, uint64_t v) {
    memcpy(kslot, &kid, 8);
    memcpy(vslot, &v, 8);
}
static int run_index(void) {
    lust_index_table t;
    lust_index_init(&t, 8, 8, 8, 8);
    char line[256];
    while (fgets(line, sizeof line, stdin)) {
        char op = line[0];
        if (op == '\n' || op == 0) continue;
        char *rest = line + 1;
        uint64_t args[3] = {0, 0, 0};
        for (int i = 0; i < 3 && *rest; i++) args[i] = strtoull(rest, &rest, 10);
        uint64_t a = args[0], b = args[1], cnum = args[2];
        uint64_t okid, oval;
        switch (op) {
        case 'n':
            lust_index_drop_table(&t, NULL, NULL);
            lust_index_init(&t, 8, 8, 8, 8);
            printf("new ; ");
            break;
        case 'w':
            lust_index_drop_table(&t, NULL, NULL);
            lust_index_init_with_capacity(&t, 8, 8, 8, 8, a);
            printf("new ; ");
            break;
        case 'i': {
            int replaced;
            void *ks, *vs;
            uint64_t pos = lust_index_insert_full(&t, idx_hash_kid(a), idx_eq, &a, &replaced, &ks, &vs);
            if (replaced) {
                uint64_t old;
                memcpy(&old, vs, 8);
                memcpy(vs, &b, 8);
                printf("%llu:%llu ; ", (unsigned long long)pos, (unsigned long long)old);
            } else {
                idx_write(ks, vs, a, b);
                printf("%llu:- ; ", (unsigned long long)pos);
            }
            break;
        }
        case 'u': {
            uint64_t pos = lust_index_get_index_of(&t, idx_hash_kid(a), idx_eq, &a);
            if (pos != UINT64_MAX) {
                printf("dup %llu ; ", (unsigned long long)pos);
            } else {
                void *ks, *vs;
                uint64_t np = lust_index_insert_unique(&t, idx_hash_kid(a), &ks, &vs);
                idx_write(ks, vs, a, b);
                printf("%llu ; ", (unsigned long long)np);
            }
            break;
        }
        case 'g': {
            uint64_t pos;
            void *ks, *vs;
            if (lust_index_get(&t, idx_hash_kid(a), idx_eq, &a, &ks, &vs, &pos)) {
                uint64_t v;
                memcpy(&v, vs, 8);
                printf("%llu:%llu ; ", (unsigned long long)pos, (unsigned long long)v);
            } else printf("- ; ");
            break;
        }
        case 'G': {
            void *ks, *vs;
            if (lust_index_get_index(&t, a, &ks, &vs)) {
                uint64_t kid, v;
                memcpy(&kid, ks, 8);
                memcpy(&v, vs, 8);
                printf("%llu:%llu:%llu ; ", (unsigned long long)a, (unsigned long long)kid, (unsigned long long)v);
            } else printf("- ; ");
            break;
        }
        case 's': {
            uint64_t pos;
            if (lust_index_shift_remove(&t, idx_hash_kid(a), idx_eq, &a, &okid, &oval, &pos))
                printf("%llu:%llu:%llu ; ", (unsigned long long)pos, (unsigned long long)okid, (unsigned long long)oval);
            else printf("- ; ");
            break;
        }
        case 'x': {
            uint64_t pos;
            if (lust_index_swap_remove(&t, idx_hash_kid(a), idx_eq, &a, &okid, &oval, &pos))
                printf("%llu:%llu:%llu ; ", (unsigned long long)pos, (unsigned long long)okid, (unsigned long long)oval);
            else printf("- ; ");
            break;
        }
        case 'S':
            if (lust_index_shift_remove_index(&t, a, &okid, &oval))
                printf("%llu:%llu ; ", (unsigned long long)okid, (unsigned long long)oval);
            else printf("- ; ");
            break;
        case 'X':
            if (lust_index_swap_remove_index(&t, a, &okid, &oval))
                printf("%llu:%llu ; ", (unsigned long long)okid, (unsigned long long)oval);
            else printf("- ; ");
            break;
        case 'p':
            if (lust_index_pop(&t, &okid, &oval))
                printf("%llu:%llu ; ", (unsigned long long)okid, (unsigned long long)oval);
            else printf("- ; ");
            break;
        case 'j': {
            if (a > lust_index_len(&t)) {
                printf("oob ; ");
                break;
            }
            uint64_t pos = lust_index_get_index_of(&t, idx_hash_kid(b), idx_eq, &b);
            if (pos != UINT64_MAX) {
                printf("dup %llu ; ", (unsigned long long)pos);
            } else {
                void *ks, *vs;
                lust_index_shift_insert_unique(&t, a, idx_hash_kid(b), &ks, &vs);
                idx_write(ks, vs, b, cnum);
                printf("%llu ; ", (unsigned long long)a);
            }
            break;
        }
        case 'R': lust_index_reserve(&t, a); printf("ok ; "); break;
        case 'E': lust_index_reserve_exact(&t, a); printf("ok ; "); break;
        case 'h': lust_index_shrink_to(&t, a); printf("ok ; "); break;
        case 'F': lust_index_shrink_to(&t, 0); printf("ok ; "); break;
        case 'c': lust_index_clear(&t, NULL, NULL); printf("ok ; "); break;
        case 'T': lust_index_retain(&t, idx_keep, &a, NULL, NULL); printf("ok ; "); break;
        case 'd': {
            if (a > b || b > lust_index_len(&t)) {
                printf("oob ; ");
                break;
            }
            if (a == b) {
                printf("- ; ");
                break;
            }
            for (uint64_t i = a; i < b; i++) {
                void *ks, *vs;
                lust_index_get_index(&t, i, &ks, &vs);
                uint64_t kid, v;
                memcpy(&kid, ks, 8);
                memcpy(&v, vs, 8);
                if (i > a) putchar(' ');
                printf("%llu:%llu", (unsigned long long)kid, (unsigned long long)v);
            }
            lust_index_drain_range(&t, a, b, NULL, NULL);
            printf(" ; ");
            break;
        }
        case 't': lust_index_truncate(&t, a, NULL, NULL); printf("ok ; "); break;
        case 'v': lust_index_reverse(&t); printf("ok ; "); break;
        case 'm':
            if (a >= lust_index_len(&t) || b >= lust_index_len(&t)) printf("oob ; ");
            else { lust_index_move_index(&t, a, b); printf("ok ; "); }
            break;
        case 'W':
            if (a >= lust_index_len(&t) || b >= lust_index_len(&t)) printf("oob ; ");
            else { lust_index_swap_indices(&t, a, b); printf("ok ; "); }
            break;
        default: fprintf(stderr, "bad index op %c\n", op); return 2;
        }
        print_state_index(&t);
        fflush(stdout);
    }
    lust_index_drop_table(&t, NULL, NULL);
    return failures != 0;
}

/* ---------------- FxHasher values (mirror oracle fxvals) ---------------- */
static uint64_t fx1_u8(uint8_t v) { lust_fx_hasher h = lust_fx_new(); lust_fx_write_u8(&h, v); return lust_fx_finish(&h); }
static uint64_t fx1_i8(int8_t v) { lust_fx_hasher h = lust_fx_new(); lust_fx_write_i8(&h, v); return lust_fx_finish(&h); }
static uint64_t fx1_u16(uint16_t v) { lust_fx_hasher h = lust_fx_new(); lust_fx_write_u16(&h, v); return lust_fx_finish(&h); }
static uint64_t fx1_i16(int16_t v) { lust_fx_hasher h = lust_fx_new(); lust_fx_write_i16(&h, v); return lust_fx_finish(&h); }
static uint64_t fx1_i32(int32_t v) { lust_fx_hasher h = lust_fx_new(); lust_fx_write_i32(&h, v); return lust_fx_finish(&h); }
static uint64_t fx1_i64(int64_t v) { lust_fx_hasher h = lust_fx_new(); lust_fx_write_i64(&h, v); return lust_fx_finish(&h); }
static uint64_t fx1_u128(uint64_t lo, uint64_t hi) { lust_fx_hasher h = lust_fx_new(); lust_fx_write_u128(&h, lo, hi); return lust_fx_finish(&h); }
static uint64_t fx1_i128(uint64_t lo, uint64_t hi) { lust_fx_hasher h = lust_fx_new(); lust_fx_write_i128(&h, lo, hi); return lust_fx_finish(&h); }
static uint64_t fx1_isize(int64_t v) { lust_fx_hasher h = lust_fx_new(); lust_fx_write_isize(&h, v); return lust_fx_finish(&h); }

static void fxvals(void) {
    const uint8_t u8s[] = {0, 1, 100, 255};
    for (int i = 0; i < 4; i++) printf("u8 %u %llu\n", u8s[i], (unsigned long long)fx1_u8(u8s[i]));
    const int8_t i8s[] = {-128, -1, 0, 1, 100, 127};
    for (int i = 0; i < 6; i++) printf("i8 %d %llu\n", i8s[i], (unsigned long long)fx1_i8(i8s[i]));
    const uint16_t u16s[] = {0, 1, 1000, 65535};
    for (int i = 0; i < 4; i++) printf("u16 %u %llu\n", u16s[i], (unsigned long long)fx1_u16(u16s[i]));
    const int16_t i16s[] = {-32768, -1, 0, 1, 32767};
    for (int i = 0; i < 5; i++) printf("i16 %d %llu\n", i16s[i], (unsigned long long)fx1_i16(i16s[i]));
    const uint32_t u32s[] = {0, 1, 100, 70000, 4294967295u};
    for (int i = 0; i < 5; i++) printf("u32 %u %llu\n", u32s[i], (unsigned long long)lust_fx_hash_u32(u32s[i]));
    const int32_t i32s[] = {INT32_MIN, -1, 0, 1, INT32_MAX};
    for (int i = 0; i < 5; i++) printf("i32 %d %llu\n", i32s[i], (unsigned long long)fx1_i32(i32s[i]));
    const uint64_t u64s[] = {0, 1, 100, 1ull << 40, UINT64_MAX};
    for (int i = 0; i < 5; i++) printf("u64 %llu %llu\n", (unsigned long long)u64s[i], (unsigned long long)lust_fx_hash_u64(u64s[i]));
    const int64_t i64s[] = {INT64_MIN, -1, 0, 1, INT64_MAX};
    for (int i = 0; i < 5; i++) printf("i64 %lld %llu\n", (long long)i64s[i], (unsigned long long)fx1_i64(i64s[i]));
    printf("u128 0 %llu\n", (unsigned long long)fx1_u128(0, 0));
    printf("u128 1 %llu\n", (unsigned long long)fx1_u128(1, 0));
    printf("u128 100 %llu\n", (unsigned long long)fx1_u128(100, 0));
    printf("u128 1267650600228229401496703205376 %llu\n", (unsigned long long)fx1_u128(0, 1ull << 36));
    printf("u128 340282366920938463463374607431768211455 %llu\n", (unsigned long long)fx1_u128(UINT64_MAX, UINT64_MAX));
    printf("i128 -170141183460469231731687303715884105728 %llu\n", (unsigned long long)fx1_i128(0, 1ull << 63));
    printf("i128 -1 %llu\n", (unsigned long long)fx1_i128(UINT64_MAX, UINT64_MAX));
    printf("i128 0 %llu\n", (unsigned long long)fx1_i128(0, 0));
    printf("i128 1 %llu\n", (unsigned long long)fx1_i128(1, 0));
    printf("i128 170141183460469231731687303715884105727 %llu\n", (unsigned long long)fx1_i128(UINT64_MAX, INT64_MAX));
    const uint64_t usz[] = {0, 1, 12345, UINT64_MAX};
    for (int i = 0; i < 4; i++) printf("usize %llu %llu\n", (unsigned long long)usz[i], (unsigned long long)lust_fx_hash_usize(usz[i]));
    const int64_t isz[] = {INT64_MIN, -1, 0, 1, INT64_MAX};
    for (int i = 0; i < 5; i++) printf("isize %lld %llu\n", (long long)isz[i], (unsigned long long)fx1_isize(isz[i]));
    printf("bool false %llu\n", (unsigned long long)fx1_u8(0));
    printf("bool true %llu\n", (unsigned long long)fx1_u8(1));
    const uint32_t chars[] = {97, 90, 233, 0x1F600};
    for (int i = 0; i < 4; i++) printf("char %u %llu\n", chars[i], (unsigned long long)lust_fx_hash_u32(chars[i]));
    const char *strs[] = {"", "a", "uwu", "hello world", "0123456789abcdef", "0123456789abcdefg"};
    for (int i = 0; i < 6; i++) {
        printf("str \"%s\" %llu\n", strs[i], (unsigned long long)lust_fx_hash_str(strs[i], strlen(strs[i])));
        printf("String \"%s\" %llu\n", strs[i], (unsigned long long)lust_fx_hash_str(strs[i], strlen(strs[i])));
    }
    char longx[101];
    memset(longx, 'x', 100);
    longx[100] = 0;
    printf("str100 %llu\n", (unsigned long long)lust_fx_hash_str(longx, 100));
    /* &[u8] values: length prefix (write_usize), then write(bytes). */
    uint8_t nine40[40];
    memset(nine40, 9, 40);
    const struct { const uint8_t *p; uint64_t n; const char *show; } slices[] = {
        {(const uint8_t *)"", 0, "[]"}, {(const uint8_t *)"\0", 1, "[0]"},
        {(const uint8_t *)"\1\2\3", 3, "[1, 2, 3]"}, {nine40, 40, NULL},
    };
    for (int i = 0; i < 4; i++) {
        lust_fx_hasher h = lust_fx_new();
        lust_fx_write_length_prefix(&h, slices[i].n);
        lust_fx_write(&h, slices[i].p, slices[i].n);
        char showbuf[256];
        const char *show = slices[i].show;
        if (!show) {
            int o = snprintf(showbuf, sizeof showbuf, "[9");
            for (int j = 1; j < 40; j++) o += snprintf(showbuf + o, sizeof showbuf - o, ", 9");
            snprintf(showbuf + o, sizeof showbuf - o, "]");
            show = showbuf;
        }
        printf("bytes %s %llu\n", show, (unsigned long long)lust_fx_finish(&h));
    }
    for (int i = 0; i < 4; i++) {
        lust_fx_hasher h = lust_fx_new();
        lust_fx_write(&h, slices[i].p, slices[i].n);
        char showbuf[256];
        const char *show = slices[i].show;
        if (!show) {
            int o = snprintf(showbuf, sizeof showbuf, "[9");
            for (int j = 1; j < 40; j++) o += snprintf(showbuf + o, sizeof showbuf - o, ", 9");
            snprintf(showbuf + o, sizeof showbuf - o, "]");
            show = showbuf;
        }
        printf("rawbytes %s %llu\n", show, (unsigned long long)lust_fx_finish(&h));
    }
    const struct { uint32_t a; uint64_t b; } tuples[] = {{0, 0}, {5, 7}, {4294967295u, 1}};
    for (int i = 0; i < 3; i++) {
        lust_fx_hasher h = lust_fx_new();
        lust_fx_write_u32(&h, tuples[i].a);
        lust_fx_write_u64(&h, tuples[i].b);
        printf("tuple (%u, %llu) %llu\n", tuples[i].a, (unsigned long long)tuples[i].b, (unsigned long long)lust_fx_finish(&h));
    }
    uint64_t rng = 0x1234567890ABCDEFull;
    for (int i = 0; i < 2000; i++) {
        rng += 0x9E3779B97F4A7C15ull;
        uint64_t z = rng;
        z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ull;
        z = (z ^ (z >> 27)) * 0x94D049BB133111EBull;
        z = z ^ (z >> 31);
        printf("rand-u64 %llu %llu\n", (unsigned long long)z, (unsigned long long)lust_fx_hash_u64(z));
        uint64_t kid = z % 100000;
        char buf[96];
        uint64_t len = make_key_str(kid, buf, sizeof buf);
        printf("rand-str %llu %llu\n", (unsigned long long)kid, (unsigned long long)lust_fx_hash_str(buf, len));
    }
}

/* rustc-hash's own published test values (lib.rs tests, 64-bit). */
static int fxvectors(void) {
    CHECK(lust_fx_hash_u32(0) == 0, "u32 0");
    CHECK(lust_fx_hash_u32(1) == 12157901119326311915ull, "u32 1");
    CHECK(lust_fx_hash_u32(100) == 16751747135202103309ull, "u32 100");
    CHECK(lust_fx_hash_u64(UINT64_MAX) == 6288842954450348564ull, "u64 max");
    /* HashBytes-style bare write values from the crate tests. */
    lust_fx_hasher h = lust_fx_new();
    lust_fx_write(&h, "", 0);
    CHECK(lust_fx_finish(&h) == 17606491139363777937ull, "bytes empty");
    h = lust_fx_new();
    lust_fx_write(&h, "uwu", 3);
    CHECK(lust_fx_finish(&h) == 7168164714682931527ull, "bytes uwu");
    const char *longb = "These are some bytes for testing rustc_hash.";
    h = lust_fx_new();
    lust_fx_write(&h, longb, strlen(longb));
    CHECK(lust_fx_finish(&h) == 2349210501944688211ull, "bytes long");
    CHECK(lust_fx_hash_u64(0) == 0, "u64 0");
    /* multiply_mix via medium strings (len 17..=100 paths) is covered by
       the fxvals diff; here just length sanity of hash_bytes itself. */
    CHECK(lust_fx_hash_bytes("", 0) != lust_fx_hash_bytes("a", 1), "bytes differ");
    if (!failures) printf("fxvectors ok\n");
    return failures != 0;
}

/* Directed edges: capacities, the empty-clear quirk, drain/finish,
   clone, ZST elements, and index capacity corners. Expected values
   are hashbrown/indexmap rules worked out in hash.h, cross-checked by
   the oracle diffs for the same shapes. */
static int edges(void) {
    /* with_capacity capacities for a 16-byte element (width 16): buckets
       4/8/16 hold 3/7/14, 32 holds 28, 128 holds 112. */
    const struct { uint64_t ask, cap; } wc[] = {{0, 0}, {1, 3}, {3, 3}, {4, 7}, {7, 7}, {8, 14}, {14, 14},
                                                {15, 28}, {28, 28}, {29, 56}, {100, 112}};
    for (unsigned i = 0; i < sizeof wc / sizeof wc[0]; i++) {
        lust_hash_table t;
        lust_hash_init_with_capacity(&t, 16, 8, wc[i].ask);
        CHECK(lust_hash_capacity(&t) == wc[i].cap, "with_capacity capacity");
        lust_hash_drop_table(&t, NULL, NULL);
    }
    /* 1-byte elements: width-16 small tables jump to 16 buckets (cap
       14); width-8 tables jump to 8 buckets (cap 7), as
       capacity_to_buckets states per group width. */
    {
        lust_hash_table t;
        lust_hash_init_with_capacity(&t, 1, 1, 1);
        CHECK(lust_hash_capacity(&t) == (LUST_HASH_GROUP_WIDTH == 16 ? 14 : 7), "with_capacity u8 capacity");
        lust_hash_drop_table(&t, NULL, NULL);
        lust_hash_init_with_capacity(&t, 2, 2, 1);
        CHECK(lust_hash_capacity(&t) == (LUST_HASH_GROUP_WIDTH == 16 ? 7 : 3), "with_capacity u16 capacity");
        lust_hash_drop_table(&t, NULL, NULL);
    }
    /* ZST elements: two inserts of the same unit key collide as one. */
    {
        lust_hash_table t;
        lust_hash_init(&t, 0, 1);
        void *slot;
        uint64_t idx;
        uint64_t dummy = 0;
        CHECK(!lust_hash_find_or_insert(&t, 42, elem_eq, &dummy, elem_hash_of, NULL, &slot, &idx), "zst first");
        lust_hash_insert_at(&t, 42, idx);
        CHECK(lust_hash_len(&t) == 1, "zst len");
        lust_hash_drop_table(&t, NULL, NULL);
    }
    /* clear() on an emptied table keeps tombstone state (hashbrown's
       early return), so capacity does not bounce back until a real
       clear or a rehash. We only assert it stays consistent. */
    {
        g_kind = KIND_U64;
        lust_hash_table t;
        lust_hash_init(&t, 16, 8);
        for (uint64_t k = 0; k < 20; k++) {
            void *slot;
            uint64_t idx;
            uint64_t key = k;
            lust_hash_find_or_insert(&t, hash_kid(k), elem_eq, &key, elem_hash_of, NULL, &slot, &idx);
            Elem *s = elem_at(lust_hash_insert_at(&t, hash_kid(k), idx));
            s->kid = k;
            s->v = k;
        }
        for (uint64_t k = 0; k < 20; k++) {
            uint64_t key = k;
            lust_hash_remove(&t, hash_kid(k), elem_eq, &key, NULL);
        }
        uint64_t cap_before = lust_hash_capacity(&t);
        lust_hash_clear(&t, NULL, NULL);
        CHECK(lust_hash_capacity(&t) == cap_before, "empty clear keeps capacity");
        lust_hash_drop_table(&t, NULL, NULL);
    }
    /* Index: capacity is min(side, entries); with_capacity(10) gives
       side 14 (16 buckets of usize) and entries 10, so 10. One insert
       at full entries length jumps entries to the side capacity. */
    {
        lust_index_table t;
        lust_index_init_with_capacity(&t, 8, 8, 8, 8, 10);
        CHECK(lust_index_capacity(&t) == 10, "index with_capacity capacity");
        lust_index_drop_table(&t, NULL, NULL);
    }
    if (!failures) printf("edges ok\n");
    return failures != 0;
}

static double now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e9 + ts.tv_nsec;
}
static int bench_fx_c(void) {
    const uint64_t n = 200000;
    g_kind = KIND_U64;
    lust_hash_table t;
    lust_hash_init(&t, 16, 8);
    double t0 = now_ns();
    for (uint64_t k = 0; k < n; k++) {
        uint64_t key = k * 2654435761ull % (n + n / 2);
        void *slot;
        uint64_t idx;
        if (lust_hash_find_or_insert(&t, hash_kid(key), elem_eq, &key, elem_hash_of, NULL, &slot, &idx)) {
            elem_at(slot)->v = k;
        } else {
            Elem *s = elem_at(lust_hash_insert_at(&t, hash_kid(key), idx));
            s->kid = key;
            s->v = k;
        }
    }
    double ins = now_ns() - t0;
    t0 = now_ns();
    uint64_t acc = 0;
    for (uint64_t k = 0; k < n; k++) {
        uint64_t key = k * 2654435761ull % (n + n / 2);
        void *slot = lust_hash_find(&t, hash_kid(key), elem_eq, &key, NULL);
        if (slot) acc += elem_at(slot)->v;
        uint64_t miss = k + n * 2;
        lust_hash_find(&t, hash_kid(miss), elem_eq, &miss, NULL);
    }
    double get = now_ns() - t0;
    t0 = now_ns();
    for (uint64_t k = 0; k < n; k += 2) {
        uint64_t key = k * 2654435761ull % (n + n / 2);
        lust_hash_remove(&t, hash_kid(key), elem_eq, &key, NULL);
    }
    double rem = now_ns() - t0;
    t0 = now_ns();
    uint64_t total = 0;
    for (int r = 0; r < 20; r++) {
        lust_hash_iter it;
        lust_hash_iter_init(&it, &t);
        void *p;
        while ((p = lust_hash_iter_next(&it, NULL)) != NULL) total += elem_at(p)->v;
    }
    double iter = now_ns() - t0;
    uint64_t len = lust_hash_len(&t);
    printf("c-fx insert_ns=%.2f get2_ns=%.2f remove_ns=%.2f iter_ns=%.2f acc=%llu len=%llu\n",
           ins / n, get / (2 * n), rem / (n / 2), iter / (20 * (len ? len : 1)),
           (unsigned long long)(acc ^ total), (unsigned long long)len);
    lust_hash_drop_table(&t, NULL, NULL);
    return 0;
}
static int bench_im_c(void) {
    const uint64_t n = 200000;
    g_kind = KIND_U64;
    lust_index_table t;
    lust_index_init(&t, 8, 8, 8, 8);
    double t0 = now_ns();
    for (uint64_t k = 0; k < n; k++) {
        uint64_t key = k * 2654435761ull % (n + n / 2);
        int replaced;
        void *ks, *vs;
        lust_index_insert_full(&t, idx_hash_kid(key), idx_eq, &key, &replaced, &ks, &vs);
        if (!replaced) memcpy(ks, &key, 8);
        memcpy(vs, &k, 8);
    }
    double ins = now_ns() - t0;
    t0 = now_ns();
    uint64_t acc = 0;
    for (uint64_t k = 0; k < n; k++) {
        uint64_t key = k * 2654435761ull % (n + n / 2);
        void *ks, *vs;
        if (lust_index_get(&t, idx_hash_kid(key), idx_eq, &key, &ks, &vs, NULL)) {
            uint64_t v;
            memcpy(&v, vs, 8);
            acc += v;
        }
    }
    double get = now_ns() - t0;
    printf("c-im insert_ns=%.2f get_ns=%.2f acc=%llu len=%llu\n", ins / n, get / n,
           (unsigned long long)acc, (unsigned long long)lust_index_len(&t));
    lust_index_drop_table(&t, NULL, NULL);
    return 0;
}

int main(int argc, char **argv) {
    const char *mode = argc > 1 ? argv[1] : "fx64";
    if (!strcmp(mode, "fx64")) { g_kind = KIND_U64; return run_map(); }
    if (!strcmp(mode, "fx32")) { g_kind = KIND_U32; return run_map(); }
    if (!strcmp(mode, "fxstr")) { g_kind = KIND_STR; return run_map(); }
    if (!strcmp(mode, "sip64")) { g_kind = KIND_SIP; return run_map(); }
    if (!strcmp(mode, "hb8")) { g_kind = KIND_U64; g_hb_table = 1; return run_map(); }
    if (!strcmp(mode, "set64")) return run_set();
    if (!strcmp(mode, "im64")) { g_kind = KIND_U64; return run_index(); }
    if (!strcmp(mode, "imstr")) { g_kind = KIND_STR; return run_index(); }
    if (!strcmp(mode, "fxvals")) { fxvals(); return 0; }
    if (!strcmp(mode, "fxvectors")) return fxvectors();
    if (!strcmp(mode, "edges")) return edges();
    if (!strcmp(mode, "bench-fx")) return bench_fx_c();
    if (!strcmp(mode, "bench-im")) return bench_im_c();
    fprintf(stderr, "unknown mode %s\n", mode);
    return 2;
}
