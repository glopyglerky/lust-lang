// K3: C11 atomics under the pinned tcc. Usage: atomics [threads=4] [iters=50000]
// Prints one PASS/FAIL line per check and a final verdict. Exit 0 iff all pass.
#include <stdatomic.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>

static int fails;
#define CHECK(name, cond) do { int ok_ = !!(cond); printf("%s %s\n", ok_ ? "PASS" : "FAIL", name); if (!ok_) fails++; } while (0)

// Exercise load/store/exchange/fetch_* and CAS for one integer width.
#define WIDTH_TEST(tag, T, val)                                                          \
    do {                                                                                 \
        _Atomic T a; atomic_init(&a, (T)0);                                              \
        atomic_store_explicit(&a, (T)(val), memory_order_relaxed);                       \
        int ok = atomic_load_explicit(&a, memory_order_relaxed) == (T)(val);             \
        atomic_store_explicit(&a, (T)(val), memory_order_seq_cst);                       \
        atomic_store_explicit(&a, (T)(val), memory_order_release);                       \
        ok = ok && atomic_load_explicit(&a, memory_order_acquire) == (T)(val);           \
        ok = ok && atomic_load_explicit(&a, memory_order_consume) == (T)(val);           \
        ok = ok && atomic_load_explicit(&a, memory_order_seq_cst) == (T)(val);           \
        T prev = atomic_exchange_explicit(&a, (T)5, memory_order_acq_rel);               \
        ok = ok && prev == (T)(val) && atomic_load(&a) == (T)5;                          \
        ok = ok && atomic_fetch_add_explicit(&a, (T)3, memory_order_relaxed) == (T)5;    \
        ok = ok && atomic_fetch_sub_explicit(&a, (T)1, memory_order_acq_rel) == (T)8;    \
        ok = ok && atomic_fetch_and_explicit(&a, (T)6, memory_order_seq_cst) == (T)7;    \
        ok = ok && atomic_fetch_or_explicit(&a, (T)1, memory_order_relaxed) == (T)6;     \
        ok = ok && atomic_fetch_xor_explicit(&a, (T)7, memory_order_relaxed) == (T)7;    \
        ok = ok && atomic_load(&a) == (T)0;                                              \
        T exp = 0;                                                                       \
        ok = ok && atomic_compare_exchange_strong_explicit(&a, &exp, (T)9,               \
                         memory_order_acq_rel, memory_order_acquire);                    \
        ok = ok && atomic_load(&a) == (T)9;                                              \
        exp = 0;                                                                         \
        ok = ok && !atomic_compare_exchange_strong_explicit(&a, &exp, (T)1,              \
                         memory_order_relaxed, memory_order_relaxed) && exp == (T)9;   \
        exp = 9; int tries = 0;                                                          \
        while (!atomic_compare_exchange_weak_explicit(&a, &exp, (T)2,                    \
                   memory_order_seq_cst, memory_order_seq_cst) && tries < 100) {         \
            exp = 9; tries++;                                                            \
        }                                                                                \
        ok = ok && atomic_load(&a) == (T)2;                                              \
        ok = ok && atomic_is_lock_free(&a);                                              \
        CHECK("width_" tag, ok);                                                         \
    } while (0)

static _Atomic uint64_t counter;
static _Atomic uint64_t cas_counter;
static int stress_iters;
static void *inc_worker(void *arg) {
    (void)arg;
    for (int i = 0; i < stress_iters; i++) {
        atomic_fetch_add_explicit(&counter, 1, memory_order_relaxed);
        uint64_t e = atomic_load_explicit(&cas_counter, memory_order_relaxed);
        while (!atomic_compare_exchange_weak_explicit(&cas_counter, &e, e + 1,
                   memory_order_acq_rel, memory_order_relaxed)) { }
    }
    return 0;
}

// Message passing: release store of flag must make data visible to acquire load.
static _Atomic int mp_flag;
static int mp_data;
static void *mp_writer(void *arg) {
    (void)arg; mp_data = 4242;
    atomic_store_explicit(&mp_flag, 1, memory_order_release); return 0;
}
static void *mp_reader(void *arg) {
    while (atomic_load_explicit(&mp_flag, memory_order_acquire) == 0) { }
    return (void *)(long)mp_data;
}

int main(int argc, char **argv) {
    int nthreads = argc > 1 ? atoi(argv[1]) : 4;
    stress_iters = argc > 2 ? atoi(argv[2]) : 50000;
#ifdef ATOMIC_BOOL_LOCK_FREE
    printf("INFO lockfree_macros bool=%d char=%d short=%d int=%d long=%d llong=%d ptr=%d\n",
           ATOMIC_BOOL_LOCK_FREE, ATOMIC_CHAR_LOCK_FREE, ATOMIC_SHORT_LOCK_FREE,
           ATOMIC_INT_LOCK_FREE, ATOMIC_LONG_LOCK_FREE, ATOMIC_LLONG_LOCK_FREE,
           ATOMIC_POINTER_LOCK_FREE);
    CHECK("lockfree_macros_present", 1);
#else
    // tcc's bundled stdatomic.h omits the ATOMIC_*_LOCK_FREE macros. The
    // is_lock_free checks in WIDTH_TEST carry the real lock-free answer.
    printf("INFO lockfree_macros ABSENT in this stdatomic.h\n");
    CHECK("lockfree_macros_present", 0);
#endif
    WIDTH_TEST("u8", uint8_t, 200);
    WIDTH_TEST("i8", int8_t, 100);
    WIDTH_TEST("u16", uint16_t, 60000);
    WIDTH_TEST("i16", int16_t, 30000);
    WIDTH_TEST("u32", uint32_t, 4000000000u);
    WIDTH_TEST("i32", int32_t, 2000000000);
    WIDTH_TEST("u64", uint64_t, 18000000000000000000ull);
    WIDTH_TEST("i64", int64_t, 9000000000000000000ll);
    { // bool + pointer widths lust needs
        _Atomic bool b; atomic_init(&b, false);
        int ok = atomic_exchange(&b, true) == false && atomic_load(&b) == true && atomic_is_lock_free(&b);
        CHECK("width_bool", ok);
        static int cells[4];
        // Pointer fetch_add is in atomic_ptr_probe.c: tcc rejects fetch ops on
        // pointer targets, and C11 fetch_add on a pointer adds bytes, not
        // elements. Lust must emit pointer atomics on the uintptr_t bit pattern.
        _Atomic(int *) p; atomic_init(&p, &cells[0]);
        int *pexp = &cells[0];
        ok = atomic_exchange(&p, &cells[1]) == &cells[0]
             && atomic_compare_exchange_strong(&p, &pexp, &cells[2]) == 0 && pexp == &cells[1]
             && atomic_load(&p) == &cells[1] && atomic_is_lock_free(&p);
        CHECK("width_ptr_load_store_xchg_cas", ok);
        _Atomic uintptr_t up; atomic_init(&up, (uintptr_t)&cells[0]);
        ok = atomic_fetch_add(&up, 2 * sizeof(int)) == (uintptr_t)&cells[0]
             && atomic_load(&up) == (uintptr_t)&cells[2];
        CHECK("width_ptr_via_uintptr_fetch_add", ok);
        _Atomic size_t s; atomic_init(&s, (size_t)7);
        CHECK("width_usize", atomic_fetch_add(&s, 1) == 7 && atomic_is_lock_free(&s));
    }
    // 128-bit atomics live in atomic128_probe.c: tcc rejects size > 8 at
    // compile time, so that case cannot share this file.
    atomic_thread_fence(memory_order_relaxed);
    atomic_thread_fence(memory_order_consume);
    atomic_thread_fence(memory_order_acquire);
    atomic_thread_fence(memory_order_release);
    atomic_thread_fence(memory_order_acq_rel);
    atomic_thread_fence(memory_order_seq_cst);
    atomic_signal_fence(memory_order_seq_cst);
    CHECK("fences_all_orders", 1);
    { // spin hint candidates named in ARCH 7.3; forms are probed in spin_probe.c.
        // tcc parses __builtin_ia32_pause as an implicit extern and then fails
        // the link, so the tcc path uses inline asm. That block cannot live in
        // this file under cc -Werror-style builds either way; keep both forms.
#if defined(__TINYC__) && (defined(__x86_64__) || defined(__i386__))
        __asm__ volatile("pause");
        CHECK("spin_hint_asm_pause", 1);
#elif defined(__x86_64__) || defined(__i386__)
        __builtin_ia32_pause();
        CHECK("spin_hint_ia32_pause", 1);
#else
        atomic_signal_fence(memory_order_seq_cst);
        CHECK("spin_hint_barrier_only", 1);
#endif
    }
    pthread_t th[8];
    if (nthreads < 2) nthreads = 2;
    if (nthreads > 8) nthreads = 8;
    for (int i = 0; i < nthreads; i++) pthread_create(&th[i], 0, inc_worker, 0);
    for (int i = 0; i < nthreads; i++) pthread_join(th[i], 0);
    uint64_t want = (uint64_t)nthreads * (uint64_t)stress_iters;
    printf("INFO stress threads=%d iters=%d counter=%llu cas=%llu want=%llu\n",
           nthreads, stress_iters, (unsigned long long)atomic_load(&counter),
           (unsigned long long)atomic_load(&cas_counter), (unsigned long long)want);
    CHECK("stress_fetch_add", atomic_load(&counter) == want);
    CHECK("stress_cas", atomic_load(&cas_counter) == want);
    pthread_t r, w; void *res = 0;
    pthread_create(&r, 0, mp_reader, 0);
    pthread_create(&w, 0, mp_writer, 0);
    pthread_join(w, 0); pthread_join(r, &res);
    CHECK("message_passing_acq_rel", (long)res == 4242);
    printf("RESULT %s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
