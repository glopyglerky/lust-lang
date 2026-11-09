// K6: C view of shim.rs exports.
#ifndef LUST_K6_SHIM_H
#define LUST_K6_SHIM_H
#include <stdint.h>
#include <stddef.h>
long shim_add(long a, long b);
int shim_catch_panic(char *out, size_t cap);
void shim_tls_set(long v);
long shim_tls_get(void);
int shim_spawn_sum(unsigned n, long *out_sum);
int shim_spawn_sum_nopanic(unsigned n, long *out_sum);
typedef int (*shim_callback)(void *env, long value);
int shim_call_callback(shim_callback cb, void *env, long n, long *out_calls);
unsigned char *shim_alloc_buffer(size_t len, unsigned char fill);
void shim_free_buffer(unsigned char *ptr, size_t len);
int shim_check_buffer(const unsigned char *ptr, size_t len, unsigned char fill);
size_t shim_fill_caller(unsigned char *ptr, size_t len, unsigned char fill);
#endif
