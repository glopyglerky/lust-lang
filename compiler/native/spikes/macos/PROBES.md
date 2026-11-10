# K3/K6 on macOS arm64: probe list

No Mac is in this environment. Nothing in this file is a measured Mac
result. `run-macos.sh` in this directory runs the list on a Mac. Send the
full log back. The "tcc source says" column comes from the pinned tinycc
source at commit 43c7708b (`third_party/tinycc`), read on Linux.

## Setup probes

| # | Probe | Why | tcc source says |
|---|---|---|---|
| M0 | `scripts/build-tcc.sh` builds tcc and libtcc for arm64 Mach-O. `tcc -v` prints arm64 Darwin. | Everything else needs the pinned build. | `configure` has a Darwin case that sets `TCC_TARGET_MACHO` (configure line 673). `arm64-gen.c`, `arm64-link.c`, and `tccmacho.c` exist. Build on macOS 14+ is unproven. |
| M1 | tcc links a hello-world exe and a `-dynamiclib`; both run; `otool -l` shows normal load commands. | Proves the Mach-O writer and dyld accept the output. | `tccmacho.c` writes Mach-O from internal ELF sections. `unwind_tables` is forced 0 for non-ELF output (tccelf.c line 94). |

## K3 probes

| # | Probe | Why | tcc source says |
|---|---|---|---|
| M2 | `spikes/k3/atomics.c` with 2, 4, and 8 threads. | Same as Linux K3: all widths, orders, CAS, fences. | Size check in tccgen.c allows sizes 1/2/4/8 only. `lib/stdatomic.c` generates the `__atomic_*_{1,2,4,8}` helpers for all targets and says size 8 is lock-free on aarch64. `lib/atomic.S` has an aarch64 block (line 1502) with load/store/CAS. Expect pass. The `ATOMIC_*_LOCK_FREE` macros are absent from tcc's `stdatomic.h` on every target. |
| M3 | Spin hint: try `__builtin_arm_yield()`, `__asm__ volatile("yield")`, and a plain compiler barrier. | Linux x86_64 uses `__builtin_ia32_pause`. ARCH 7.3 allows a barrier as fallback. | No pause or yield builtin in the tcc source (`grep ia32_pause` finds nothing; the x86 header path is what made it work on Linux). Expect: compiler barrier only. Measure nothing; a barrier is acceptable. |
| M4 | `spikes/k3/atomic128_probe.c`. | Expect a compile error, as on Linux. | tccgen.c rejects `size > 8` for atomic builtins on all targets. Expect the same error. Rust has no stable `AtomicU128`, so this is informational. |
| M5 | `spikes/k3/i128_probe.c`. | Expect `__int128` rejection. | No `__int128` token; `include/tccdefs.h` lines 182-186 fake `__int128_t` as a 16-byte struct so system headers parse. Expect rejection. The two-word helpers in `u128.h` are the path on both targets. |
| M6 | `spikes/k3/u128_test.c` against the rustc oracle vectors. | The helpers must match rustc on arm64 too. Endianness is little-endian on both targets. | Pure C, no tcc feature use. Expect pass. One risk: struct return-by-value ABI for a 16-byte struct (AAPCS64: registers x0/x1). The test returns `lust_u128` by value on every call, so a mismatch fails loudly. |
| M7 | `spikes/k3/align.c` plus `attr_variants.c`. | Packed layout, `_Alignas`, 4 MiB static and heap alignment. | `MAX_ALIGN` is 16 on arm64 (arm64-gen.c line 39), same as x86_64. Attribute handling is in shared tccgen.c code, so expect the same ignores as Linux: `__attribute__((packed))` ignored, `#pragma pack` works, `__attribute__((aligned))` on structs ignored, member `_Alignas` works. Mach-O section alignment goes through `exact_log2p1(s->sh_addralign)` in tccmacho.c; Mach-O caps section alignment at 2^15 in the section header field. A 4 MiB-aligned Mach-O section is impossible in that field, so `align_static_4MiB` may fail structurally. Heap `posix_memalign` must carry the 4 MiB case. |
| M8 | TLS: `tls_exe.c`, `tls_so.c` as a dylib, `tls_linked.c`, `tls_dlopen.c`. Also `otool -l` for `__thread_data`/`__thread_bss` sections and `nm` for TLV descriptors. | The daemon layout rule (no C TLS outside the exe) comes from the Linux alias bug. The Mac rule needs its own facts. | arm64-gen.c lines 471-487 emit ELF local-exec TLS for non-PE: `mrs x, tpidr_el0` plus `R_AARCH64_TLSLE_ADD_TPREL_*` relocs. macOS does not use ELF TPREL; it uses TLV descriptors (`__thread_vars`, `tlv_get_addr`). tccmacho.c has no TLV or `__thread_data` handling (the only `STT_TLS` hit maps a TLS symbol to a plain `N_SECT` symbol, line 1061). Expect `_Thread_local` to be plain data (shared across threads) or a link error on Mach-O. That is the same failure class as Linux .so TLS, so the pthread-key rule is expected to stand. Measure it; do not assume. |
| M9 | `spikes/k3/pthread_key.c`. | The chosen portable TLS mechanism. | Plain libc. Expect pass, destructors included. |

## K6 probes

| # | Probe | Why | tcc source says |
|---|---|---|---|
| M10 | Build `spikes/k6/shim.rs` as staticlib (`.a`) and cdylib (`.dylib`) with the installed rustc. Run `spikes/k6/main.c` through: tcc CLI + staticlib, libtcc + staticlib, tcc CLI + cdylib, libtcc + cdylib, system `cc` (clang) + tcc object + staticlib, `cc` + cdylib, and dlopen of the dylib from a tcc exe. | The decisive K6 question: does a caught Rust panic survive each final link? | The Linux root cause is in tcc's handling of `.eh_frame`/`.init_array`/TLS relocs from rustc objects. Mach-O unwind data is `__eh_frame` plus compact unwind; tccmacho forces `unwind_tables = 0` for Mach-O and has no compact-unwind synthesis for foreign objects. Expect the tcc-linker staticlib catch to fail at least as badly as on Linux. The cdylib path puts unwind under dyld, as on Linux. Measure all seven. |
| M11 | Daemon layout: dylib-walled shim plus overlay exe via libtcc, using `install_name`/`-rpath` forms that dyld accepts from tcc output. | `link_with_base` in src/tcc.rs emits `-Wl,-rpath=<dir>`. dyld wants `LC_RPATH`; confirm tcc's Mach-O writer emits it and that a tcc exe finds `libbase.dylib`. | Check `otool -l` output for `LC_RPATH`, `LC_ID_DYLIB`, and `LC_LOAD_DYLIB`. If rpath does not survive, the dev fallback is `DYLD_LIBRARY_PATH` in the launcher, which is acceptable for dev only. |
| M12 | Rust shim inside a dylib + `_Thread_local` in lust C code in the same dylib. | Confirms the wall rule: Rust TLS in the dylib must work through dyld TLV while lust C TLS stays banned. | Rust `thread_local!` in a cdylib uses real dyld TLV and is independent of tcc. Expect pass. |
| M13 | Link timing with `k6/gen_big.py` and the same paths as M10. | Dev loop cost of the system-linker fallback on Mac. | Linux numbers are in RESULTS.md. dyld/clang link of a 200-export staticlib is expected in the same band. Measure, interleaved, median of 5. |

## Fallbacks if a probe fails

- M2 fails (atomics broken on arm64): emit atomics as calls to `libatomic`
  style helpers compiled by the system compiler into the shim dylib, one
  function per width and op. The C backend then needs no `_Atomic` on
  arm64. Cost: a call per atomic op, dev only.
- M7 static 4 MiB fails: keep statics at the highest working alignment and
  route every Rust alignment over 64 KiB to the heap over-align path,
  which align.c already probes.
- M8 worse than Linux (exe TLS also broken): the pthread-key design
  already assumes this. No design change.
- M10 cdylib paths fail: stop. No Mac dev story exists until either tcc
  is patched (we pin the source, so a patch is possible) or final links
  go through `cc` for everything, daemon overlay included.
