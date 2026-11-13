/* End-to-end C test for liblust_shim.so.
 *
 * Built with the pinned tcc and linked against the cdylib, the link
 * path spikes/RESULTS.md K6-4 proves. It exercises every export,
 * including a forced shim panic, threads that call the shim at once,
 * and the one callback export.
 *
 * Prints PASS and FAIL lines, `ORACLE key=value` lines for the values
 * src/bin/oracle.rs computes directly in Rust (test.sh diffs them),
 * and one RESULT line. Exit code is the fail count clamped to 1.
 *
 * Args: <tzif-file> <zone-name> <stat-file> <work-dir>
 * No _Thread_local anywhere (spikes K3 rule for tcc-built code).
 */
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <poll.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/prctl.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#include "lust_shim.h"

static int fails;

#define CHECK(name, cond)                                                            \
    do {                                                                             \
        int ok_ = !!(cond);                                                          \
        printf("%s %s\n", ok_ ? "PASS" : "FAIL", name);                              \
        if (!ok_) fails++;                                                           \
    } while (0)

/* The shared probe pattern. oracle.rs builds it with the same rule. */
static uint8_t pat[10000];
static void make_pattern(void) {
    for (size_t i = 0; i < sizeof pat; i++) pat[i] = (uint8_t)((i * 31 + 7) % 256);
}

static const char *g_tzif;
static const char *g_zone;
static const char *g_statfile;
static const char *g_workdir;

/* Prints the last shim error for a failed check. */
static void info_err(const char *what) {
    char msg[256] = {0};
    lust_shim_last_error(msg, sizeof msg);
    printf("INFO %s last_error='%s'\n", what, msg);
}

/* Reads a whole small file with stdio. Returns bytes read or -1. */
static long read_file(const char *path, uint8_t *buf, size_t cap) {
    FILE *f = fopen(path, "rb");
    if (!f) return -1;
    size_t n = fread(buf, 1, cap, f);
    fclose(f);
    return (long)n;
}

/* Joins dir and name into a static path buffer. One use at a time. */
static char *join(const char *a, const char *b) {
    static char buf[1024];
    snprintf(buf, sizeof buf, "%s/%s", a, b);
    return buf;
}

static void test_core_and_alloc(void) {
    CHECK("abi_version", lust_shim_abi_version() == 1);

    int32_t st = lust_shim_panic_for_test();
    char msg[256] = {0};
    size_t need = lust_shim_last_error(msg, sizeof msg);
    printf("INFO panic status=%d msg='%s'\n", st, msg);
    CHECK("panic_is_status", st == LUST_SHIM_ERR_PANIC);
    CHECK("panic_message", strcmp(msg, "lust shim forced panic 42") == 0 && need == 25);

    /* A bad handle is INVALID, and the shim stays usable after it. */
    uint64_t v = 0;
    CHECK("bad_handle_invalid", lust_shim_xxh3_digest64(999999, &v) == LUST_SHIM_ERR_INVALID);

    uint8_t *p = NULL;
    CHECK("alloc", lust_shim_alloc(100, 16, &p) == 0 && p && ((uintptr_t)p % 16) == 0);
    memset(p, 0x5a, 100);
    uint8_t *big = NULL;
    CHECK("realloc_grow", lust_shim_realloc(p, 100, 16, 5000, &big) == 0 && big && big[0] == 0x5a && big[99] == 0x5a);
    lust_shim_dealloc(big, 5000, 16);

    uint8_t *z = NULL;
    int zeros = 1;
    if (lust_shim_alloc_zeroed(256, 64, &z) == 0 && z) {
        for (int i = 0; i < 256; i++) zeros = zeros && z[i] == 0;
    } else {
        zeros = 0;
    }
    CHECK("alloc_zeroed", zeros && ((uintptr_t)z % 64) == 0);
    lust_shim_dealloc(z, 256, 64);

    CHECK("alloc_bad_align", lust_shim_alloc(10, 3, &p) == LUST_SHIM_ERR_INVALID);

    /* buf_free on a zeroed buf and on NULL does nothing. */
    LustShimBuf empty = {0, 0, 0};
    lust_shim_buf_free(&empty);
    lust_shim_buf_free(NULL);
    CHECK("buf_free_empty", empty.ptr == NULL);
}

static void test_xxhash(void) {
    uint64_t h = 0, lo = 0, hi = 0;
    lust_shim_xxh3_64(NULL, 0, &h);
    printf("ORACLE xx64_empty=%" PRIu64 "\n", h);
    lust_shim_xxh3_64((const uint8_t *)"hello lust shim", 15, &h);
    printf("ORACLE xx64_hello=%" PRIu64 "\n", h);
    lust_shim_xxh3_128((const uint8_t *)"hello lust shim", 15, &lo, &hi);
    printf("ORACLE xx128_hello_lo=%" PRIu64 "\n", lo);
    printf("ORACLE xx128_hello_hi=%" PRIu64 "\n", hi);
    lust_shim_xxh3_64(pat, sizeof pat, &h);
    printf("ORACLE xx64_pattern=%" PRIu64 "\n", h);
    lust_shim_xxh3_128(pat, sizeof pat, &lo, &hi);
    printf("ORACLE xx128_pattern_lo=%" PRIu64 "\n", lo);
    printf("ORACLE xx128_pattern_hi=%" PRIu64 "\n", hi);

    uint64_t sh = 0;
    int ok = lust_shim_xxh3_new(&sh) == 0 && sh != 0;
    ok = ok && lust_shim_xxh3_update(sh, pat, 3000) == 0;
    ok = ok && lust_shim_xxh3_update(sh, pat + 3000, 1) == 0;
    ok = ok && lust_shim_xxh3_update(sh, pat + 3001, sizeof pat - 3001) == 0;
    CHECK("xxh3_stream_new_update", ok);
    lust_shim_xxh3_digest64(sh, &h);
    printf("ORACLE xx_stream64=%" PRIu64 "\n", h);
    lust_shim_xxh3_digest128(sh, &lo, &hi);
    printf("ORACLE xx_stream128_lo=%" PRIu64 "\n", lo);
    printf("ORACLE xx_stream128_hi=%" PRIu64 "\n", hi);
    CHECK("xxh3_free", lust_shim_xxh3_free(sh) == 0);
    CHECK("xxh3_free_again_invalid", lust_shim_xxh3_free(sh) == LUST_SHIM_ERR_INVALID);
}

static void test_gzip(void) {
    LustShimBuf gz = {0, 0, 0};
    int ok = lust_shim_gzip_encode(pat, sizeof pat, 1, &gz) == 0;
    CHECK("gzip_encode", ok && gz.len > 0);
    uint64_t h = 0;
    lust_shim_xxh3_64(gz.ptr, gz.len, &h);
    printf("ORACLE gzip_len=%zu\n", gz.len);
    printf("ORACLE gzip_xx64=%" PRIu64 "\n", h);

    LustShimBuf back = {0, 0, 0};
    ok = lust_shim_gzip_decode_multi(gz.ptr, gz.len, &back) == 0;
    uint64_t bh = 0;
    lust_shim_xxh3_64(back.ptr, back.len, &bh);
    printf("ORACLE gzip_roundtrip_xx64=%" PRIu64 "\n", bh);
    CHECK("gzip_roundtrip_bytes", ok && back.len == sizeof pat && memcmp(back.ptr, pat, sizeof pat) == 0);

    /* Two members decode as one stream (MultiGzDecoder). */
    LustShimBuf gz2 = {0, 0, 0}, both = {0, 0, 0}, multi = {0, 0, 0};
    const char *tail = "hello lust shim";
    lust_shim_gzip_encode((const uint8_t *)tail, 15, 6, &gz2);
    uint8_t *cat = malloc(gz.len + gz2.len);
    memcpy(cat, gz.ptr, gz.len);
    memcpy(cat + gz.len, gz2.ptr, gz2.len);
    int mok = lust_shim_gzip_decode_multi(cat, gz.len + gz2.len, &multi) == 0;
    CHECK("gzip_decode_multi_member",
          mok && multi.len == sizeof pat + 15 && memcmp(multi.ptr, pat, sizeof pat) == 0 &&
              memcmp(multi.ptr + sizeof pat, tail, 15) == 0);
    free(cat);
    (void)both;

    LustShimBuf junk = {0, 0, 0};
    int32_t st = lust_shim_gzip_decode_multi((const uint8_t *)"not gzip", 8, &junk);
    if (st == 0) info_err("gzip_bad");
    CHECK("gzip_bad_input_fails", st != 0);
    CHECK("gzip_bad_level", lust_shim_gzip_encode(pat, 4, 10, &junk) == LUST_SHIM_ERR_INVALID);

    lust_shim_buf_free(&gz);
    lust_shim_buf_free(&gz2);
    lust_shim_buf_free(&back);
    lust_shim_buf_free(&multi);
    lust_shim_buf_free(&junk);
    CHECK("gzip_buf_freed_zeroed", gz.ptr == NULL && gz.len == 0);
}

static uint64_t f64_bits(double v) {
    uint64_t b;
    memcpy(&b, &v, 8);
    return b;
}

static void test_bigint(void) {
    double f = 0;
    int32_t has = -1;
    lust_shim_bigint_parse_to_f64((const uint8_t *)"123456789abcdef", 15, 16, &f, &has);
    printf("ORACLE bigint_hex_f64_bits=%" PRIu64 "\n", f64_bits(f));
    CHECK("bigint_parse_f64_has", has == 1);

    LustShimBuf dec = {0, 0, 0};
    lust_shim_bigint_parse_to_decimal((const uint8_t *)"ffffffffffffffff", 16, 16, &dec, &has);
    printf("ORACLE bigint_dec=%.*s\n", (int)dec.len, (const char *)dec.ptr);
    CHECK("bigint_decimal_text", has == 1 && dec.len == 20 && memcmp(dec.ptr, "18446744073709551615", 20) == 0);
    lust_shim_buf_free(&dec);

    lust_shim_bigint_pow_i64_to_f64(3, 40, &f, &has);
    printf("ORACLE bigint_pow_bits=%" PRIu64 "\n", f64_bits(f));
    CHECK("bigint_pow_has", has == 1);

    has = -1;
    lust_shim_bigint_parse_to_f64((const uint8_t *)"12x", 3, 10, &f, &has);
    printf("ORACLE bigint_bad_has=%d\n", has);
    CHECK("bigint_bad_digits_none", has == 0);
}

static void oracle_dt(const char *key, const LustShimDateTime *dt) {
    printf("ORACLE %s=%d-%02d-%02d %02d:%02d:%02d.%d\n", key, dt->year, dt->month, dt->day,
           dt->hour, dt->minute, dt->second, dt->subsec_nanos);
}

static void test_jiff(void) {
    static uint8_t zone_data[1 << 16];
    long n = read_file(g_tzif, zone_data, sizeof zone_data);
    CHECK("tzif_file_read", n > 0);

    uint64_t tz = 0;
    int ok = lust_shim_jiff_tz_from_tzif((const uint8_t *)g_zone, strlen(g_zone), zone_data,
                                         (size_t)n, &tz) == 0;
    if (!ok) info_err("tz_from_tzif");
    CHECK("tz_from_tzif", ok && tz != 0);

    LustShimDateTime dt = {0};
    lust_shim_jiff_tz_to_datetime(tz, 1709651045, 123456789, &dt);
    oracle_dt("tz_dt1", &dt);
    lust_shim_jiff_tz_to_datetime(tz, -1, 0, &dt);
    oracle_dt("tz_dt_neg", &dt);

    LustShimBuf clock1 = {0, 0, 0}, clock2 = {0, 0, 0};
    lust_shim_jiff_format_clock(tz, 1709651045, 123456789, LUST_SHIM_CLOCK_STATUS, &clock1);
    printf("ORACLE tz_clock_status=%.*s\n", (int)clock1.len, (const char *)clock1.ptr);
    lust_shim_jiff_format_clock(tz, 1709651045, 123456789, LUST_SHIM_CLOCK_MILLIS, &clock2);
    printf("ORACLE tz_clock_millis=%.*s\n", (int)clock2.len, (const char *)clock2.ptr);
    CHECK("clock_bad_kind", lust_shim_jiff_format_clock(tz, 0, 0, 9, &clock1) == LUST_SHIM_ERR_INVALID);
    lust_shim_buf_free(&clock1);
    lust_shim_buf_free(&clock2);

    uint64_t local = 0;
    ok = lust_shim_jiff_tz_local(&local) == 0;
    lust_shim_jiff_tz_to_datetime(local, 1709651045, 123456789, &dt);
    oracle_dt("tz_local_dt1", &dt);
    CHECK("tz_local", ok && local != 0);
    lust_shim_jiff_tz_free(local);

    uint64_t utc = 0;
    lust_shim_jiff_tz_utc(&utc);
    lust_shim_jiff_tz_to_datetime(utc, 0, 0, &dt);
    CHECK("tz_utc_epoch", dt.year == 1970 && dt.month == 1 && dt.day == 1 && dt.hour == 0);
    lust_shim_jiff_tz_free(utc);

    uint64_t bad = 0;
    int32_t st = lust_shim_jiff_tz_from_tzif((const uint8_t *)"Bad", 3, (const uint8_t *)"junk", 4, &bad);
    CHECK("tz_bad_tzif_fails", st != 0);
    CHECK("tz_range_fails", lust_shim_jiff_tz_to_datetime(tz, (int64_t)4e18, 0, &dt) == LUST_SHIM_ERR);
    CHECK("tz_free", lust_shim_jiff_tz_free(tz) == 0);
    CHECK("tz_use_after_free", lust_shim_jiff_tz_to_datetime(tz, 0, 0, &dt) == LUST_SHIM_ERR_INVALID);
}

struct dir_walk {
    int calls;
    int seen_a;
    int seen_b;
    int stop_after; /* 0 means never */
};

static int on_dirent(void *env, uint64_t ino, uint32_t type, const uint8_t *name, size_t len) {
    struct dir_walk *w = (struct dir_walk *)env;
    (void)ino;
    (void)type;
    w->calls++;
    if (len == 5 && memcmp(name, "a.txt", 5) == 0) w->seen_a = 1;
    if (len == 5 && memcmp(name, "b.txt", 5) == 0) w->seen_b = 1;
    if (w->stop_after > 0 && w->calls >= w->stop_after) return 7;
    return 0;
}

/* Parses linux_dirent64 records and looks for want by name. */
static int dirent_buf_has(const uint8_t *buf, size_t n, const char *want) {
    size_t off = 0, wlen = strlen(want);
    while (off + 19 <= n) {
        uint16_t reclen = 0;
        memcpy(&reclen, buf + off + 16, 2);
        if (reclen < 19 || off + reclen > n) return 0;
        const char *name = (const char *)(buf + off + 19);
        size_t room = reclen - 19;
        if (strlen(name) < room && strlen(name) == wlen && memcmp(name, want, wlen) == 0) return 1;
        off += reclen;
    }
    return 0;
}

static void test_fd(void) {
    /* Stat family on the known fixture file. */
    int32_t fd = -1;
    int ok = lust_shim_open((const uint8_t *)g_statfile, strlen(g_statfile), O_RDONLY | O_CLOEXEC, 0, &fd) == 0;
    if (!ok) info_err("open_statfile");
    CHECK("open", ok && fd >= 0);
    LustShimStat st;
    memset(&st, 0, sizeof st);
    lust_shim_fstat(fd, &st);
    printf("ORACLE stat_size=%" PRId64 "\n", st.size);
    printf("ORACLE stat_mode=%" PRIu32 "\n", st.mode);
    printf("ORACLE stat_ino=%" PRIu64 "\n", st.ino);
    printf("ORACLE stat_mtime=%" PRId64 "\n", st.mtime_sec);

    uint8_t buf[64] = {0};
    size_t n = 0;
    ok = lust_shim_read(fd, buf, sizeof buf, &n) == 0;
    CHECK("read_known_file", ok && n == (size_t)st.size && memcmp(buf, "lust shim stat probe", 20) == 0);
    CHECK("close", lust_shim_close(fd) == 0);
    CHECK("close_again_ebadf", lust_shim_close(fd) == EBADF);
    CHECK("read_closed_ebadf", lust_shim_read(fd, buf, 1, &n) == EBADF);

    LustShimStat lst;
    memset(&lst, 0, sizeof lst);
    ok = lust_shim_lstat((const uint8_t *)g_statfile, strlen(g_statfile), &lst) == 0;
    CHECK("lstat_matches_fstat", ok && lst.ino == st.ino && lst.size == st.size);
    memset(&lst, 0, sizeof lst);
    ok = lust_shim_statat(LUST_SHIM_AT_FDCWD, (const uint8_t *)g_statfile, strlen(g_statfile),
                          0x100 /* AT_SYMLINK_NOFOLLOW */, &lst) == 0;
    CHECK("statat", ok && lst.ino == st.ino);

    LustShimStatx sx;
    memset(&sx, 0, sizeof sx);
    /* STATX_TYPE|STATX_MTIME = 0x800, AT_SYMLINK_NOFOLLOW = 0x100. */
    lust_shim_statx(LUST_SHIM_AT_FDCWD, (const uint8_t *)g_statfile, strlen(g_statfile), 0x100, 0x800, &sx);
    printf("ORACLE statx_mode=%" PRIu32 "\n", sx.mode);
    printf("ORACLE statx_mtime=%" PRId64 "\n", sx.mtime_sec);
    CHECK("statx_mask_has_mtime", (sx.mask & 0x800) == 0x800);

    /* Missing file gives the errno, NUL in a path gives EINVAL. */
    char *missing = join(g_workdir, "no-such-file");
    CHECK("open_missing_enoent",
          lust_shim_open((const uint8_t *)missing, strlen(missing), O_RDONLY, 0, &fd) == ENOENT);
    CHECK("open_nul_path_einval", lust_shim_open((const uint8_t *)"a\0b", 3, O_RDONLY, 0, &fd) == EINVAL);

    /* accessat: R_OK=4 with AT_EACCESS=0x200. */
    CHECK("accessat_read",
          lust_shim_accessat(LUST_SHIM_AT_FDCWD, (const uint8_t *)g_statfile, strlen(g_statfile), 4, 0x200) == 0);
    CHECK("accessat_missing",
          lust_shim_accessat(LUST_SHIM_AT_FDCWD, (const uint8_t *)missing, strlen(missing), 4, 0) == ENOENT);

    /* Create and write through openat relative to the work dir fd. */
    int32_t dirfd = -1;
    ok = lust_shim_open((const uint8_t *)g_workdir, strlen(g_workdir),
                        O_RDONLY | O_DIRECTORY | O_CLOEXEC, 0, &dirfd) == 0;
    CHECK("open_dir", ok && dirfd >= 0);
    int32_t wfd = -1;
    ok = lust_shim_openat(dirfd, (const uint8_t *)"a.txt", 5, O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC, 0644, &wfd) == 0;
    size_t wrote = 0;
    ok = ok && lust_shim_write(wfd, (const uint8_t *)"alpha", 5, &wrote) == 0 && wrote == 5;
    CHECK("openat_write", ok);
    lust_shim_close(wfd);
    ok = lust_shim_openat(dirfd, (const uint8_t *)"b.txt", 5, O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC, 0644, &wfd) == 0;
    ok = ok && lust_shim_write(wfd, (const uint8_t *)"beta", 4, &wrote) == 0;
    lust_shim_close(wfd);
    CHECK("openat_second_file", ok);

    /* utimensat moves mtime and leaves atime out with UTIME_OMIT. */
    char saved[1024];
    snprintf(saved, sizeof saved, "%s", join(g_workdir, "a.txt"));
    LustShimTimestamps times = {0, LUST_SHIM_UTIME_OMIT, 1700000000, 0};
    ok = lust_shim_utimensat(LUST_SHIM_AT_FDCWD, (const uint8_t *)saved, strlen(saved), &times, 0) == 0;
    memset(&lst, 0, sizeof lst);
    lust_shim_lstat((const uint8_t *)saved, strlen(saved), &lst);
    CHECK("utimensat_mtime", ok && lst.mtime_sec == 1700000000);

    /* mknodat makes a fifo (S_IFIFO), the stat type mode proves. */
    ok = lust_shim_mknodat(dirfd, (const uint8_t *)"pipe1", 5, S_IFIFO, 0644, 0) == 0;
    if (!ok) info_err("mknodat");
    memset(&lst, 0, sizeof lst);
    lust_shim_statat(dirfd, (const uint8_t *)"pipe1", 5, 0x100, &lst);
    CHECK("mknodat_fifo", ok && (lst.mode & S_IFMT) == S_IFIFO);

    LustShimStatvfs vfs;
    memset(&vfs, 0, sizeof vfs);
    ok = lust_shim_statvfs((const uint8_t *)g_workdir, strlen(g_workdir), &vfs) == 0;
    CHECK("statvfs", ok && vfs.bsize > 0 && vfs.blocks > 0);

    /* Directory reads: raw dirent batch, then callback walks. */
    LustShimBuf dents = {0, 0, 0};
    ok = lust_shim_read_dirent(dirfd, 8192, &dents) == 0;
    CHECK("read_dirent_has_files",
          ok && dirent_buf_has(dents.ptr, dents.len, "a.txt") && dirent_buf_has(dents.ptr, dents.len, "b.txt"));
    lust_shim_buf_free(&dents);
    lust_shim_close(dirfd);

    int32_t fd2 = -1;
    lust_shim_open((const uint8_t *)g_workdir, strlen(g_workdir), O_RDONLY | O_DIRECTORY | O_CLOEXEC, 0, &fd2);
    struct dir_walk walk = {0, 0, 0, 0};
    int64_t calls = 0;
    int32_t cb_status = -1;
    ok = lust_shim_readdir_each(fd2, on_dirent, &walk, &calls, &cb_status) == 0;
    CHECK("readdir_each_all", ok && cb_status == 0 && calls == walk.calls && walk.seen_a && walk.seen_b && calls >= 5);
    lust_shim_close(fd2);

    int32_t fd3 = -1;
    lust_shim_open((const uint8_t *)g_workdir, strlen(g_workdir), O_RDONLY | O_DIRECTORY | O_CLOEXEC, 0, &fd3);
    struct dir_walk stop = {0, 0, 0, 3};
    calls = 0;
    cb_status = 0;
    ok = lust_shim_readdir_each(fd3, on_dirent, &stop, &calls, &cb_status) == 0;
    /* The shim must not call back after the first non-zero status. */
    CHECK("readdir_each_stops", ok && calls == 3 && stop.calls == 3 && cb_status == 7);
    lust_shim_close(fd3);

    /* Pipes and poll. */
    int32_t pfd[2] = {-1, -1};
    ok = lust_shim_pipe2(O_CLOEXEC | O_NONBLOCK, pfd) == 0;
    CHECK("pipe2", ok && pfd[0] >= 0 && pfd[1] >= 0);
    LustShimPollFd pollset[1] = {{pfd[0], POLLIN, 0}};
    int32_t pn = -1;
    ok = lust_shim_poll(pollset, 1, 0, &pn) == 0;
    CHECK("poll_empty_timeout", ok && pn == 0 && (pollset[0].revents & POLLIN) == 0);
    size_t wn = 0;
    lust_shim_write(pfd[1], (const uint8_t *)"xy", 2, &wn);
    pollset[0].revents = 0;
    ok = lust_shim_poll(pollset, 1, 1000, &pn) == 0;
    uint8_t got[4] = {0};
    size_t rn = 0;
    lust_shim_read(pfd[0], got, sizeof got, &rn);
    CHECK("poll_readable_then_read", ok && pn == 1 && (pollset[0].revents & POLLIN) != 0 && rn == 2 && memcmp(got, "xy", 2) == 0);
    /* Borrowed raw fd: poll on C stdin must not fail with EBADF. */
    LustShimPollFd rawset[1] = {{0, POLLIN, 0}};
    CHECK("poll_borrowed_raw_fd", lust_shim_poll(rawset, 1, 0, &pn) == 0);
    uint32_t fl = 0;
    CHECK("fcntl_getfl_borrowed_stdout", lust_shim_fcntl_getfl(1, &fl) == 0);
    LustShimWinsize ws;
    memset(&ws, 0, sizeof ws);
    CHECK("winsize_on_pipe_enotty", lust_shim_tcgetwinsize(pfd[0], &ws) == ENOTTY);
    int32_t wst = lust_shim_tcgetwinsize(1, &ws);
    printf("INFO winsize_stdout status=%d col=%u (a pipe gives ENOTTY)\n", wst, ws.col);
    CHECK("winsize_stdout_ok_or_enotty", wst == 0 || wst == ENOTTY || wst == EINVAL);
    lust_shim_close(pfd[0]);
    lust_shim_close(pfd[1]);
}

/* Waits for one child and returns its wait status, or -1. */
static int reap(pid_t pid) {
    int status = 0;
    pid_t got;
    do {
        got = waitpid(pid, &status, 0);
    } while (got < 0 && errno == EINTR);
    return got == pid ? status : -1;
}

static void msleep(long ms) {
    struct timespec ts = {ms / 1000, (ms % 1000) * 1000000L};
    nanosleep(&ts, NULL);
}

static void test_proc(void) {
    int32_t pid = 0, ppid = 0;
    lust_shim_getpid(&pid);
    lust_shim_getppid(&ppid);
    CHECK("getpid_matches_c", pid == (int32_t)getpid());
    CHECK("getppid_matches_c", ppid == (int32_t)getppid());
    CHECK("test_kill_self", lust_shim_test_kill(pid) == 0);

    /* kill: a paused child ends by SIGTERM from the shim. */
    pid_t child = fork();
    if (child == 0) {
        for (;;) pause();
    }
    int ok = child > 0 && lust_shim_kill((int32_t)child, SIGTERM) == 0;
    int ws = reap(child);
    CHECK("kill_child_sigterm", ok && WIFSIGNALED(ws) && WTERMSIG(ws) == SIGTERM);
    /* The reaped pid is gone (a fresh reuse in this instant is the one
     * race; a failure here with a live reused pid would show at once). */
    int32_t gone = lust_shim_test_kill((int32_t)child);
    printf("INFO test_kill_reaped status=%d (ESRCH=%d)\n", gone, ESRCH);
    CHECK("test_kill_reaped_esrch", gone == ESRCH);

    /* waitid WNOWAIT: after it, waitpid still gets the exit code. */
    child = fork();
    if (child == 0) _exit(7);
    int32_t wst = 0;
    do {
        wst = lust_shim_waitid_exited_nowait((int32_t)child);
    } while (wst == EINTR);
    ws = reap(child);
    CHECK("waitid_nowait_keeps_status", wst == 0 && WIFEXITED(ws) && WEXITSTATUS(ws) == 7);

    /* Parent-death signal, verified inside the child with prctl. */
    child = fork();
    if (child == 0) {
        if (lust_shim_set_parent_death_signal(SIGKILL) != 0) _exit(3);
        int sig = 0;
        if (prctl(PR_GET_PDEATHSIG, (unsigned long)&sig) != 0) _exit(4);
        _exit(sig == SIGKILL ? 0 : 5);
    }
    ws = reap(child);
    CHECK("set_parent_death_signal", WIFEXITED(ws) && WEXITSTATUS(ws) == 0);

    /* dup2 to stdio in a child: its stdout write goes to /dev/null. */
    child = fork();
    if (child == 0) {
        int32_t nfd = -1;
        if (lust_shim_open((const uint8_t *)"/dev/null", 9, O_WRONLY, 0, &nfd) != 0) _exit(3);
        if (lust_shim_dup2_stdout(nfd) != 0) _exit(4);
        if (lust_shim_dup2_stderr(nfd) != 0) _exit(5);
        ssize_t w = write(1, "MUST_NOT_APPEAR\n", 16);
        _exit(w == 16 ? 0 : 6);
    }
    ws = reap(child);
    CHECK("dup2_stdio_child", WIFEXITED(ws) && WEXITSTATUS(ws) == 0);

    LustShimRlimit lim;
    memset(&lim, 0, sizeof lim);
    ok = lust_shim_getrlimit(LUST_SHIM_RLIMIT_NOFILE, &lim) == 0;
    struct rlimit crl;
    memset(&crl, 0, sizeof crl);
    getrlimit(RLIMIT_NOFILE, &crl);
    CHECK("getrlimit_nofile_matches_c",
          ok && lim.current == (uint64_t)crl.rlim_cur && lim.maximum == (uint64_t)crl.rlim_max);
    CHECK("setrlimit_same_value", lust_shim_setrlimit(LUST_SHIM_RLIMIT_NOFILE, &lim) == 0);
    memset(&lim, 0, sizeof lim);
    ok = lust_shim_getrlimit(LUST_SHIM_RLIMIT_AS, &lim) == 0 &&
         lust_shim_getrlimit(LUST_SHIM_RLIMIT_DATA, &lim) == 0;
    CHECK("getrlimit_as_data", ok);
    CHECK("getrlimit_bad_resource", lust_shim_getrlimit(99, &lim) == LUST_SHIM_ERR_INVALID);

    int32_t thp = -1;
    ok = lust_shim_prctl_get_thp_disable(&thp) == 0 && (thp == 0 || thp == 1);
    ok = ok && lust_shim_prctl_set_thp_disable(thp) == 0;
    if (!ok) info_err("prctl_thp");
    CHECK("prctl_thp_round_trip_same_value", ok);

    /* Block SIGWINCH with C, unblock it through the shim (nix SigSet). */
    sigset_t block, cur;
    sigemptyset(&block);
    sigaddset(&block, SIGWINCH);
    pthread_sigmask(SIG_BLOCK, &block, NULL);
    int32_t sigs[1] = {SIGWINCH};
    int ust = lust_shim_thread_unblock_signals(sigs, 1);
    sigemptyset(&cur);
    pthread_sigmask(SIG_SETMASK, NULL, &cur);
    CHECK("thread_unblock_signals", ust == 0 && !sigismember(&cur, SIGWINCH));
}

struct worker_arg {
    int tid;
    int ok;
    uint64_t want_hash;
};

/* Each thread hashes, stats, and makes its own shim error. The last
 * error text must be the one this thread made, which proves Rust TLS
 * in the cdylib stays per thread under C pthreads. */
static void *worker(void *p) {
    struct worker_arg *a = (struct worker_arg *)p;
    uint64_t h = 0;
    int ok = lust_shim_xxh3_64(pat, sizeof pat, &h) == 0 && h == a->want_hash;
    LustShimStat st;
    memset(&st, 0, sizeof st);
    ok = ok && lust_shim_fstat(1, &st) == 0;
    const char *needle;
    int32_t st2;
    if (a->tid % 2 == 0) {
        st2 = lust_shim_xxh3_digest64(888000 + (uint64_t)a->tid, &h);
        needle = "xxh3";
    } else {
        LustShimDateTime dt;
        memset(&dt, 0, sizeof dt);
        st2 = lust_shim_jiff_tz_to_datetime(888000 + (uint64_t)a->tid, 0, 0, &dt);
        needle = "time zone";
    }
    char msg[256] = {0};
    lust_shim_last_error(msg, sizeof msg);
    ok = ok && st2 == LUST_SHIM_ERR_INVALID && strstr(msg, needle) != NULL;
    a->ok = ok;
    return NULL;
}

static void *panic_worker(void *p) {
    int *out = (int *)p;
    *out = lust_shim_panic_for_test() == LUST_SHIM_ERR_PANIC;
    return NULL;
}

static void test_threads(void) {
    uint64_t want = 0;
    lust_shim_xxh3_64(pat, sizeof pat, &want);
    enum { NT = 8 };
    pthread_t th[NT];
    struct worker_arg args[NT];
    for (int i = 0; i < NT; i++) {
        args[i].tid = i;
        args[i].ok = 0;
        args[i].want_hash = want;
        pthread_create(&th[i], NULL, worker, &args[i]);
    }
    int all = 1;
    for (int i = 0; i < NT; i++) {
        pthread_join(th[i], NULL);
        all = all && args[i].ok;
    }
    CHECK("pthreads_hash_fstat_tls_error", all);

    /* The main thread error text survives a panic on another thread. */
    uint64_t h = 0;
    lust_shim_xxh3_digest64(777001, &h);
    int panic_ok = 0;
    pthread_t t;
    pthread_create(&t, NULL, panic_worker, &panic_ok);
    pthread_join(t, NULL);
    char msg[256] = {0};
    lust_shim_last_error(msg, sizeof msg);
    CHECK("panic_on_c_thread", panic_ok);
    CHECK("main_last_error_untouched", strstr(msg, "xxh3") != NULL);
}

/* Polls until pred on the flag value holds or about a second passes. */
static int flag_wait(uint64_t flag, int32_t want) {
    for (int i = 0; i < 200; i++) {
        int32_t v = -1;
        lust_shim_signal_flag_get(flag, &v);
        if (v == want) return 1;
        msleep(5);
    }
    return 0;
}

struct next_arg {
    uint64_t queue;
    int32_t signal;
    int32_t has;
    int32_t status;
};

static void *next_worker(void *p) {
    struct next_arg *a = (struct next_arg *)p;
    a->status = lust_shim_signals_next(a->queue, &a->signal, &a->has);
    return NULL;
}

static void test_signals(void) {
    /* Flag register: SIGUSR1 must set the flag, not end the process. */
    uint64_t flag = 0;
    int ok = lust_shim_signal_flag_new(0, &flag) == 0;
    int32_t v = -1;
    lust_shim_signal_flag_get(flag, &v);
    ok = ok && v == 0;
    ok = ok && lust_shim_signal_register_flag(SIGUSR1, flag) == 0;
    raise(SIGUSR1);
    ok = ok && flag_wait(flag, 1);
    CHECK("signal_flag_register_raise", ok);
    CHECK("signal_flag_set", lust_shim_signal_flag_set(flag, 0) == 0 && flag_wait(flag, 0));

    /* Conditional default, in children: true flag ends by SIGUSR2,
     * a cleared flag lets the child run on. */
    pid_t child = fork();
    if (child == 0) {
        uint64_t f = 0;
        lust_shim_signal_flag_new(1, &f);
        lust_shim_signal_register_conditional_default(SIGUSR2, f);
        raise(SIGUSR2);
        _exit(9); /* reached only if the default action did not run */
    }
    int ws = reap(child);
    CHECK("conditional_default_true_dies", WIFSIGNALED(ws) && WTERMSIG(ws) == SIGUSR2);
    child = fork();
    if (child == 0) {
        uint64_t f = 0;
        lust_shim_signal_flag_new(1, &f);
        lust_shim_signal_register_conditional_default(SIGUSR2, f);
        lust_shim_signal_flag_set(f, 0);
        raise(SIGUSR2);
        msleep(50);
        _exit(0);
    }
    ws = reap(child);
    CHECK("conditional_default_cleared_lives", WIFEXITED(ws) && WEXITSTATUS(ws) == 0);
    lust_shim_signal_flag_free(flag);

    /* Iterator: wait batch, add, pending batch, next, close. */
    int32_t sigs[1] = {SIGUSR1};
    uint64_t q = 0;
    ok = lust_shim_signals_new(sigs, 1, &q) == 0;
    if (!ok) info_err("signals_new");
    CHECK("signals_new", ok && q != 0);
    raise(SIGUSR1);
    int32_t batch[8] = {0};
    size_t bn = 99;
    ok = lust_shim_signals_wait_batch(q, batch, 8, &bn) == 0;
    CHECK("signals_wait_batch", ok && bn >= 1 && batch[0] == SIGUSR1);
    ok = lust_shim_signals_add(q, SIGUSR2) == 0;
    raise(SIGUSR2);
    bn = 0;
    for (int i = 0; i < 200 && bn == 0; i++) {
        lust_shim_signals_pending_batch(q, batch, 8, &bn);
        if (bn == 0) msleep(5);
    }
    CHECK("signals_add_pending_batch", ok && bn >= 1 && batch[0] == SIGUSR2);

    raise(SIGUSR1);
    int32_t got = 0, has = 0;
    ok = lust_shim_signals_next(q, &got, &has) == 0;
    CHECK("signals_next_queued", ok && has == 1 && got == SIGUSR1);

    /* Close from the main thread ends a next blocked on another one. */
    struct next_arg na = {q, 0, -1, -99};
    pthread_t t;
    pthread_create(&t, NULL, next_worker, &na);
    msleep(100);
    lust_shim_signals_close(q);
    pthread_join(t, NULL);
    CHECK("signals_close_unblocks_next", na.status == 0 && na.has == 0);
    CHECK("signals_free", lust_shim_signals_free(q) == 0);

    /* Emulate default: a child with a flag handler dies by SIGUSR1. */
    child = fork();
    if (child == 0) {
        uint64_t f = 0;
        lust_shim_signal_flag_new(0, &f);
        lust_shim_signal_register_flag(SIGUSR1, f);
        lust_shim_signal_emulate_default(SIGUSR1);
        _exit(9);
    }
    ws = reap(child);
    CHECK("emulate_default_dies_by_signal", WIFSIGNALED(ws) && WTERMSIG(ws) == SIGUSR1);
}

/* Picks the watch masks from <sys/inotify.h> shape by raw values, so
 * the header set is not needed here beyond fcntl flags. */
static void test_watch(void) {
    const int32_t IN_NON = 0x800, IN_CLEX = 0x80000;
    const uint32_t IN_CREATE_V = 0x100, IN_MODIFY_V = 0x2;
    int32_t ifd = -1;
    int ok = lust_shim_inotify_init1(IN_NON | IN_CLEX, &ifd) == 0;
    if (!ok) info_err("inotify_init1");
    CHECK("inotify_init1", ok && ifd >= 0);
    int32_t wd = -1;
    ok = lust_shim_inotify_add_watch(ifd, (const uint8_t *)g_workdir, strlen(g_workdir),
                                     IN_CREATE_V | IN_MODIFY_V, &wd) == 0;
    CHECK("inotify_add_watch", ok && wd >= 0);
    char *fresh = join(g_workdir, "fresh.txt");
    FILE *f = fopen(fresh, "w");
    if (f) {
        fputs("fresh", f);
        fclose(f);
    }
    LustShimPollFd ps[1] = {{ifd, POLLIN, 0}};
    int32_t pn = 0;
    ok = lust_shim_poll(ps, 1, 2000, &pn) == 0 && pn == 1;
    uint8_t ev[4096] = {0};
    size_t rn = 0;
    ok = ok && lust_shim_read(ifd, ev, sizeof ev, &rn) == 0 && rn >= 16;
    int32_t ewd = 0;
    uint32_t mask = 0;
    memcpy(&ewd, ev, 4);
    memcpy(&mask, ev + 4, 4);
    printf("INFO inotify wd=%d mask=%#x bytes=%zu\n", ewd, mask, rn);
    CHECK("inotify_event_read", ok && ewd == wd && (mask & (IN_CREATE_V | IN_MODIFY_V)) != 0);
    CHECK("inotify_rm_watch", lust_shim_inotify_rm_watch(ifd, wd) == 0);
    lust_shim_close(ifd);

    /* fanotify needs CAP_SYS_ADMIN. EPERM is a valid result here: the
     * export ran and the errno came back. On a host with the cap, the
     * full mark and event read must pass. */
    int32_t fan = -1;
    int32_t fst = lust_shim_fanotify_init(0x1 | 0x2 /* CLOEXEC|NONBLOCK */, O_RDONLY | O_CLOEXEC, &fan);
    printf("INFO fanotify_init status=%d fd=%d (EPERM=%d without CAP_SYS_ADMIN)\n", fst, fan, EPERM);
    if (fst == 0) {
        int mok = lust_shim_fanotify_mark(fan, 0x1 | 0x8 /* ADD|ONLYDIR */, 0x100 | 0x2 /* CREATE|MODIFY */,
                                          LUST_SHIM_AT_FDCWD, (const uint8_t *)g_workdir,
                                          strlen(g_workdir), 1) == 0;
        char *fresh2 = join(g_workdir, "fresh2.txt");
        f = fopen(fresh2, "w");
        if (f) {
            fputs("fresh2", f);
            fclose(f);
        }
        LustShimPollFd fps[1] = {{fan, POLLIN, 0}};
        int32_t fn2 = 0;
        mok = mok && lust_shim_poll(fps, 1, 2000, &fn2) == 0 && fn2 == 1;
        uint8_t fev[4096] = {0};
        size_t frn = 0;
        mok = mok && lust_shim_read(fan, fev, sizeof fev, &frn) == 0 && frn >= 24;
        if (mok) {
            int32_t event_fd = 0;
            memcpy(&event_fd, fev + 16, 4);
            if (event_fd >= 0) close(event_fd);
        }
        CHECK("fanotify_mark_and_event", mok && fev[4] == 3);
        lust_shim_close(fan);
    } else {
        CHECK("fanotify_init_eperm_without_cap", fst == EPERM || fst == EACCES);
    }

    /* A non-fanotify fd in fanotify_mark gives EINVAL. */
    int32_t plain[2] = {-1, -1};
    lust_shim_pipe2(O_CLOEXEC, plain);
    CHECK("fanotify_mark_wrong_fd_einval",
          lust_shim_fanotify_mark(plain[0], 1, 0x100, LUST_SHIM_AT_FDCWD, NULL, 0, 0) == EINVAL);
    lust_shim_close(plain[0]);
    lust_shim_close(plain[1]);

    int32_t htype = 0, mount = 0;
    LustShimBuf handle = {0, 0, 0};
    int32_t hst = lust_shim_name_to_handle_at(LUST_SHIM_AT_FDCWD, (const uint8_t *)g_statfile,
                                              strlen(g_statfile), 0, &htype, &mount, &handle);
    printf("INFO name_to_handle_at status=%d type=%d mount=%d len=%zu (EOPNOTSUPP=%d)\n",
           hst, htype, mount, handle.len, EOPNOTSUPP);
    CHECK("name_to_handle_at_ok_or_unsupported_fs",
          (hst == 0 && handle.len > 0) || hst == EOPNOTSUPP || hst == ENOTSUP);
    lust_shim_buf_free(&handle);
}

static void test_jemalloc(void) {
    int32_t enabled = -1;
    lust_shim_jemalloc_enabled(&enabled);
    printf("INFO jemalloc_enabled=%d\n", enabled);
    CHECK("jemalloc_enabled_answers", enabled == 0 || enabled == 1);
    if (enabled == 1) {
        uint32_t arenas = 0, own = 0;
        int ok = lust_shim_jemalloc_opt_narenas(&arenas) == 0 && arenas >= 1;
        ok = ok && lust_shim_jemalloc_thread_arena_get(&own) == 0 && own < arenas;
        CHECK("jemalloc_arenas_read", ok);
        if (arenas > 1) {
            uint32_t other = (own + 1) % arenas, back = 9999;
            ok = lust_shim_jemalloc_thread_arena_set(other) == 0;
            lust_shim_jemalloc_thread_arena_get(&back);
            uint8_t *p = NULL;
            ok = ok && back == other && lust_shim_alloc(128, 16, &p) == 0 && p != NULL;
            lust_shim_dealloc(p, 128, 16);
            ok = ok && lust_shim_jemalloc_thread_arena_set(own) == 0;
            CHECK("jemalloc_thread_arena_assign", ok);
        }
        CHECK("jemalloc_layout", lust_shim_jemalloc_layout() == 0);
        uint8_t *p = NULL;
        int aok = lust_shim_alloc(4096, 4194304, &p) == 0 && p && ((uintptr_t)p % 4194304) == 0;
        printf("INFO alloc_4mib_align ptr=%p\n", (void *)p);
        CHECK("alloc_4mib_alignment", aok);
        lust_shim_dealloc(p, 4096, 4194304);
    } else {
        uint32_t arena = 0;
        CHECK("jemalloc_controls_unsupported",
              lust_shim_jemalloc_thread_arena_get(&arena) == LUST_SHIM_ERR_UNSUPPORTED &&
                  lust_shim_jemalloc_thread_arena_set(0) == LUST_SHIM_ERR_UNSUPPORTED &&
                  lust_shim_jemalloc_opt_narenas(&arena) == LUST_SHIM_ERR_UNSUPPORTED &&
                  lust_shim_jemalloc_layout() == LUST_SHIM_ERR_UNSUPPORTED);
    }
}

int main(int argc, char **argv) {
    setvbuf(stdout, NULL, _IOLBF, 0);
    if (argc != 5) {
        printf("usage: test_shim <tzif-file> <zone-name> <stat-file> <work-dir>\n");
        return 2;
    }
    g_tzif = argv[1];
    g_zone = argv[2];
    g_statfile = argv[3];
    g_workdir = argv[4];
    make_pattern();

    test_core_and_alloc();
    test_xxhash();
    test_gzip();
    test_bigint();
    test_jiff();
    test_fd();
    test_proc();
    test_threads();
    test_signals();
    test_watch();
    test_jemalloc();

    printf("RESULT %s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
