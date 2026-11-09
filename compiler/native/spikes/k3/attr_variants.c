// K3: attribute placement variants. Informational: prints layouts, no verdict.
#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
struct P1 { uint8_t a; uint32_t b; uint16_t c; uint64_t d; } __attribute__((packed));
struct __attribute__((packed)) P2 { uint8_t a; uint32_t b; uint16_t c; uint64_t d; };
struct P3 { uint8_t a; uint32_t b __attribute__((packed)); uint16_t c __attribute__((packed)); uint64_t d __attribute__((packed)); };
#pragma pack(push, 1)
struct P4 { uint8_t a; uint32_t b; uint16_t c; uint64_t d; };
#pragma pack(pop)
struct __attribute__((aligned(32))) A1 { char c; };
struct A2 { char c; } __attribute__((aligned(32)));
typedef struct { char c; } __attribute__((aligned(32))) A3;
#ifdef __TINYC__
typedef _Alignas(32) struct { char c; } A4;
typedef _Alignas(16) struct { uint64_t lo, hi; } U1;
#else
typedef struct { char c; } A4;
typedef struct { uint64_t lo, hi; } U1;
#endif
struct A5 { _Alignas(32) char c; };
struct __attribute__((aligned(16))) U2 { uint64_t lo, hi; };
struct U3 { char tag; struct U2 v; };
struct U4 { char tag; U1 v; };
int main(void) {
    printf("P1 size=%zu offd=%zu | P2 size=%zu offd=%zu | P3 size=%zu offd=%zu | P4pragma size=%zu offd=%zu\n",
        sizeof(struct P1), offsetof(struct P1, d), sizeof(struct P2), offsetof(struct P2, d),
        sizeof(struct P3), offsetof(struct P3, d), sizeof(struct P4), offsetof(struct P4, d));
    printf("A1 size=%zu align=%zu | A2 size=%zu align=%zu | A3 size=%zu align=%zu | A4 size=%zu align=%zu | A5 size=%zu align=%zu\n",
        sizeof(struct A1), _Alignof(struct A1), sizeof(struct A2), _Alignof(struct A2),
        sizeof(A3), _Alignof(A3), sizeof(A4), _Alignof(A4), sizeof(struct A5), _Alignof(struct A5));
    printf("U1 size=%zu align=%zu U4off=%zu size=%zu | U2 size=%zu align=%zu U3off=%zu size=%zu\n",
        sizeof(U1), _Alignof(U1), offsetof(struct U4, v), sizeof(struct U4),
        sizeof(struct U2), _Alignof(struct U2), offsetof(struct U3, v), sizeof(struct U3));
    _Alignas(128) char l128[8];
    _Alignas(256) char l256[8];
    _Alignas(512) char l512[8];
    _Alignas(1024) char l1024[8];
    _Alignas(2048) char l2048[8];
    _Alignas(4096) char l4096[8];
    printf("locals mod: 128->%llu 256->%llu 512->%llu 1024->%llu 2048->%llu 4096->%llu\n",
        (unsigned long long)((uintptr_t)l128 & 127), (unsigned long long)((uintptr_t)l256 & 255),
        (unsigned long long)((uintptr_t)l512 & 511), (unsigned long long)((uintptr_t)l1024 & 1023),
        (unsigned long long)((uintptr_t)l2048 & 2047), (unsigned long long)((uintptr_t)l4096 & 4095));
    return 0;
}
