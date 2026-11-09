/* The std platform crate's runtime (docs/ARCH.md sections 3.1 and 3.7,
   package W-SHIM): the C behind the `sys_*` intrinsics of src/ty/lang.rs,
   called from library/std/src/platform.rs and its modules. A `&[u8]`
   argument (SigTy::Bytes) arrives as a pointer and a length; `&mut
   String` and `&mut Vec<u8>` arrive as pointers to the runtime structs.
   OS errors return as errno values, as std's `io::Error::last_os_error`
   reads them.

   Declarations here are in every unit; bodies (sys.c) only in the
   runtime unit (LUST_IMPL), like deps.h. */
#ifndef LUST_RT_SYS_H
#define LUST_RT_SYS_H

/* struct stat time fields by name: Linux spells them st_atim and kin,
   Darwin st_atimespec and kin, and only Darwin's stat carries the
   birth time in the stat itself (Linux reads it through statx). The
   sys*.c files below read times through these spellings. */
#ifdef __APPLE__
#define LUST_ST_ATIME(st) (st).st_atimespec
#define LUST_ST_MTIME(st) (st).st_mtimespec
#define LUST_ST_CTIME(st) (st).st_ctimespec
#define LUST_ST_BTIME(st) (st).st_birthtimespec
#else
#define LUST_ST_ATIME(st) (st).st_atim
#define LUST_ST_MTIME(st) (st).st_mtim
#define LUST_ST_CTIME(st) (st).st_ctim
#endif

/* Standard streams: writes go through the C stdio streams `print!` and
   `eprint!` use (stderr is flushed after each write, as Rust's stderr
   is unbuffered). */
int64_t lust_sys_write_std(int32_t fd, const uint8_t *p, uint64_t len);
int32_t lust_sys_flush_std(int32_t fd);

/* The text of a `fmt::Arguments` value (fmt.h), formatted now, as a
   Vec<u8> (std's `io::Write::write_fmt`). A fmt impl error panics as
   std's does. */
lust_Vec lust_sys_fmt_arguments_bytes(lust_Arguments args);

/* Byte views of text (OsStr, Path): a `&[u8]` return comes back as a
   lust_str (cgen rebuilds the slice value from its ptr and len). */
lust_str lust_sys_str_as_bytes(lust_str s);
lust_str lust_sys_bytes_as_str(const uint8_t *p, uint64_t len);
lust_String lust_sys_string_from_utf8_unchecked(lust_Vec v);
uint64_t lust_sys_utf8_valid_up_to(const uint8_t *p, uint64_t len);
void lust_sys_utf8_lossy(const uint8_t *p, uint64_t len, lust_String *out);

/* Environment: getenv into a Vec<u8>; 0 when unset or the key has NUL. */
uint8_t lust_sys_getenv(const uint8_t *k, uint64_t klen, lust_Vec *out);

/* Environment and process state: `environ` and `argv` entries each
   with a NUL terminator; getcwd, chdir, readlink. Path arguments with
   a NUL byte return -1 (no syscall); other failures return errno. */
void lust_sys_environ(lust_Vec *out);
void lust_sys_argv(lust_Vec *out);
int32_t lust_sys_getcwd(lust_Vec *out);
int32_t lust_sys_chdir(const uint8_t *p, uint64_t len);
int32_t lust_sys_symlink(const uint8_t *a, uint64_t alen, const uint8_t *b, uint64_t blen);
int32_t lust_sys_readlink(const uint8_t *p, uint64_t len, lust_Vec *out);

/* Files and directories (std::fs): read a whole file into the Vec,
   write (create or truncate, 0666; -2 for a zero-length write), mkdir
   (op 0), unlink (op 1), rmdir (op 2), rename, stat or lstat as 8-byte
   fields, realpath, the names of a directory, and remove_dir_all. Same
   returns as above: 0, errno, or -1 for a path with a NUL byte. */
int32_t lust_sys_read_file(const uint8_t *p, uint64_t len, lust_Vec *out);
int32_t lust_sys_write_file(const uint8_t *p, uint64_t len, const uint8_t *d, uint64_t dlen);
int32_t lust_sys_path_op(const uint8_t *p, uint64_t len, int32_t op);
int32_t lust_sys_rename(const uint8_t *a, uint64_t alen, const uint8_t *b, uint64_t blen);
int32_t lust_sys_stat(const uint8_t *p, uint64_t len, uint8_t follow, lust_Vec *out);
int32_t lust_sys_realpath(const uint8_t *p, uint64_t len, lust_Vec *out);
int32_t lust_sys_read_dir(const uint8_t *p, uint64_t len, lust_Vec *out);
int32_t lust_sys_remove_dir_all(const uint8_t *p, uint64_t len);

/* The statx birth time behind `Metadata::created` (sys.c):
   1 with the seconds and nanoseconds in the out args when the
   filesystem reports one, else 0. */
int32_t lust_sys_btime_path(const char *path, int32_t follow, int64_t *sec, int64_t *nsec);
int32_t lust_sys_btime_fd(int32_t fd, int64_t *sec, int64_t *nsec);

uint32_t lust_sys_getpid(void);

/* The running executable's path appended to the Vec (`env::current_exe`):
   /proc/self/exe on Linux, _NSGetExecutablePath on Darwin (no NUL
   appended). Returns 0 or the errno of the failed call, as std's
   last_os_error forms read it. */
int32_t lust_sys_current_exe(lust_Vec *out);

/* The OS text of an error code, as std's `error_string` (strerror). */
void lust_sys_os_error_text(int32_t code, lust_String *out);

#ifdef LUST_IMPL
#include "sys.c"
#endif

#endif
