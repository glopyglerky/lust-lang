// K3: alignment and packed layout under tcc. ts-rust needs a 4 MiB-aligned
// allocation (jemalloc_layout.rs); the dev allocator must honor align 4 MiB.
// Checks static/local/struct alignment, packed layout, and aligned allocs.
// Note: tcc predefines __STDC_VERSION__=199901L, so glibc hides the C11
// aligned_alloc declaration. Without the explicit declaration below, tcc
// truncates its return value to int. Lust runtime C must declare it too.
#define _GNU_SOURCE 1
#include <stdio.h>
void *aligned_alloc(size_t alignment, size_t size);
#include <stdlib.h>
#include <stdint.h>
#include <stddef.h>
#include <string.h>
static int fails;
#define CHECK(name, cond) do { int ok_ = !!(cond); printf("%s %s\n", ok_ ? "PASS" : "FAIL", name); if (!ok_) fails++; } while (0)

_Alignas(16) static uint64_t pair16[2];
_Alignas(64) static char cache_line[64];
_Alignas(4096) static char page[4096];
_Alignas(4194304) static char huge[16];
typedef struct { uint64_t lo, hi; } U128;
typedef struct { char tag; _Alignas(16) U128 v; char tail; } WithAligned;
typedef struct { uint8_t a; uint32_t b; uint16_t c; uint64_t d; } __attribute__((packed)) Packed;
#pragma pack(push, 1)
typedef struct { uint8_t a; uint32_t b; uint16_t c; uint64_t d; } PackedPragma;
#pragma pack(pop)
typedef struct { uint8_t a; uint32_t b; uint16_t c; uint64_t d; } Natural;
typedef struct { char c; } __attribute__((aligned(32))) Aligned32;
typedef struct { _Alignas(32) char c; } AlignasMember32;
// 16-byte aligned u128 shape that works in tcc: _Alignas on the first member.
typedef struct { _Alignas(16) uint64_t lo; uint64_t hi; } U128A16;
typedef struct { char tag; U128A16 v; } WithU128A16;

static int aligned_ptr(const void *p, size_t a) { return ((uintptr_t)p & (a - 1)) == 0; }

int main(void) {
    printf("INFO alignof U128=%zu WithAligned size=%zu align=%zu off_v=%zu U128A16 size=%zu align=%zu\n",
           _Alignof(U128), sizeof(WithAligned), _Alignof(WithAligned), offsetof(WithAligned, v),
           sizeof(U128A16), _Alignof(U128A16));
    printf("INFO alignof pair16_addr_mod16=%llu page_mod4096=%llu huge_mod4MiB=%llu\n",
           (unsigned long long)((uintptr_t)pair16 & 15),
           (unsigned long long)((uintptr_t)page & 4095),
           (unsigned long long)((uintptr_t)huge & 4194303));
    CHECK("align_static_16", aligned_ptr(pair16, 16));
    CHECK("align_static_64", aligned_ptr(cache_line, 64));
    CHECK("align_static_4096", aligned_ptr(page, 4096));
    CHECK("align_static_4MiB", aligned_ptr(huge, 4194304));
    // Rust u128 in a struct: field at 16-byte offset, struct size multiple of 16.
    CHECK("align_struct_u128_field", offsetof(WithAligned, v) == 16 && sizeof(WithAligned) == 48
          && _Alignof(WithAligned) == 16);
    CHECK("align_struct_u128a16", sizeof(U128A16) == 16 && _Alignof(U128A16) == 16
          && offsetof(WithU128A16, v) == 16 && sizeof(WithU128A16) == 32);
#ifdef __STDC_VERSION__
    printf("INFO __STDC_VERSION__=%ld\n", __STDC_VERSION__);
#endif
    printf("INFO packed_attr size=%zu off_d=%zu packed_pragma size=%zu off_d=%zu natural size=%zu off_d=%zu\n",
           sizeof(Packed), offsetof(Packed, d), sizeof(PackedPragma), offsetof(PackedPragma, d),
           sizeof(Natural), offsetof(Natural, d));
    CHECK("packed_attr_layout", sizeof(Packed) == 15 && offsetof(Packed, b) == 1
          && offsetof(Packed, c) == 5 && offsetof(Packed, d) == 7);
    CHECK("packed_pragma_layout", sizeof(PackedPragma) == 15 && offsetof(PackedPragma, b) == 1
          && offsetof(PackedPragma, c) == 5 && offsetof(PackedPragma, d) == 7);
    { // unaligned access through a packed struct must read back the value
        PackedPragma pk; memset(&pk, 0, sizeof pk);
        pk.a = 0xaa; pk.b = 0xdeadbeefu; pk.c = 0x1234; pk.d = 0x0102030405060708ull;
        CHECK("packed_pragma_access", pk.a == 0xaa && pk.b == 0xdeadbeefu && pk.c == 0x1234
              && pk.d == 0x0102030405060708ull);
        unsigned char raw[15]; memcpy(raw, &pk, 15);
        CHECK("packed_pragma_bytes", raw[0] == 0xaa && raw[1] == 0xef && raw[4] == 0xde
              && raw[5] == 0x34 && raw[7] == 0x08 && raw[14] == 0x01);
    }
    printf("INFO attr_aligned32 size=%zu align=%zu alignas_member32 size=%zu align=%zu\n",
           sizeof(Aligned32), _Alignof(Aligned32), sizeof(AlignasMember32), _Alignof(AlignasMember32));
    CHECK("attr_aligned_32", sizeof(Aligned32) == 32 && _Alignof(Aligned32) == 32);
    CHECK("alignas_member_32", sizeof(AlignasMember32) == 32 && _Alignof(AlignasMember32) == 32);
    { // locals with big alignment. tcc does not realign the stack, so above
      // the ABI's 16 bytes the result depends on the entry stack address and
      // varies between runs. Only align_local_16 is a stable guarantee.
        _Alignas(16) char local16[16];
        _Alignas(64) char local64[64];
        _Alignas(256) char local256[16];
        _Alignas(512) char local512[16];
        _Alignas(4096) char local_page[16];
        printf("INFO local mods 16=%llu 64=%llu 256=%llu 512=%llu 4096=%llu\n",
               (unsigned long long)((uintptr_t)local16 & 15),
               (unsigned long long)((uintptr_t)local64 & 63),
               (unsigned long long)((uintptr_t)local256 & 255),
               (unsigned long long)((uintptr_t)local512 & 511),
               (unsigned long long)((uintptr_t)local_page & 4095));
        CHECK("align_local_16", aligned_ptr(local16, 16));
        CHECK("align_local_64", aligned_ptr(local64, 64));
        CHECK("align_local_256", aligned_ptr(local256, 256));
        CHECK("align_local_512", aligned_ptr(local512, 512));
        CHECK("align_local_4096", aligned_ptr(local_page, 4096));
    }
    { // heap: posix_memalign and aligned_alloc at 4 MiB (the ts-rust case)
        void *p = 0;
        int rc = posix_memalign(&p, 4194304, 4194304);
        int ok = rc == 0 && p && aligned_ptr(p, 4194304);
        if (ok) { memset(p, 0x5a, 4096); ok = ((unsigned char *)p)[0] == 0x5a; }
        printf("INFO posix_memalign_4MiB rc=%d ptr=%p\n", rc, p);
        CHECK("alloc_posix_memalign_4MiB", ok);
        free(p);
        void *q = aligned_alloc(4194304, 4194304);
        printf("INFO aligned_alloc_4MiB ptr=%p\n", q);
        CHECK("alloc_aligned_alloc_4MiB", q && aligned_ptr(q, 4194304));
        free(q);
        // Over-align pattern lust's runtime can use with plain malloc: store base.
        size_t align = 4194304, size = 1024;
        void *raw = malloc(size + align + sizeof(void *));
        uintptr_t up = ((uintptr_t)raw + sizeof(void *) + align - 1) & ~(uintptr_t)(align - 1);
        ((void **)up)[-1] = raw;
        CHECK("alloc_manual_overalign_4MiB", raw && aligned_ptr((void *)up, align));
        free(((void **)up)[-1]);
    }
    printf("RESULT %s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
