//! Go: pprof/pprof.go (package `pprof`), and the parts of the Go standard
//! library package `runtime/pprof` (go1.27.1, the oracle's toolchain) that
//! it calls. go1.27.1 changed only the parts that the port does not run (the
//! record loop of `writeHeapProto` and the text form of `writeHeap`).
//!
//! `BeginProfiling` and `ProfileSession.Stop` serve `tscCompilation` and
//! `tscBuildCompilation` (execute/execute_tsc.rs). `CPUProfiler`,
//! `SaveHeapProfile`, `SaveAllocProfile` and `RunGC` serve the language
//! server (lsp/server.rs) and the API session (api/session_p1.rs).
//!
//! PORT: Go `runtime/pprof` builds its profiles from samples that the Go
//! runtime takes: CPU samples on SIGPROF, and allocation samples every
//! `runtime.MemProfileRate` bytes. The port has no such sampler, so it has
//! no samples. It writes the profiles as `runtime/pprof` writes a profile
//! with no samples: the gzip-compressed `profile.proto` message with the
//! sample types, the period, the times, the executable mappings of
//! `/proc/self/maps` with their GNU build IDs, and the string table. It
//! writes no `Sample`, `Location` or `Function` message. The directory,
//! the file names, the file order and the printed lines are Go's. The pid,
//! the times, the mappings and the profile bytes never match Go, like the
//! memory statistics (execute/tsc/statistics.rs `read_mem_stats`).
//!
//! PORT: Go `runtime.GC()` has nothing to do in Rust (no garbage
//! collector), so `run_gc` is a no-op. The `os` calls of the language
//! server functions (`MkdirAll`, `Create`, `Remove`) give Go's error texts
//! (`mkdir_all`, `path_error`).

use std::cell::RefCell;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::mem::ManuallyDrop;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::FileExt;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::rustc_hash::FxHashMap;

use crate::core::go_panic;
// PORT: the paths are port forms of Go strings (see
// `scanner_util::GO_STRING_MARKER`); the OS gets their Go bytes (`os_path`).
use crate::frontend::vfs::osvfs::{};
use crate::gostd::{GoError, errors};

// Go: pprof/pprof.go:15 ProfileSession
// PORT: `cpu_file` is `None` after Go `p.cpuFile.Close()`. `cpu_profile`
// is the state of the Go `runtime/pprof` goroutine `profileWriter` (see
// `start_cpu_profile`).

// Go: pprof/pprof.go:23 BeginProfiling
// BeginProfiling starts CPU and memory profiling, writing the profiles to the specified directory.
// PORT: Go `panic(err)` is `go_panic` with the Go error text.


// PORT: Go `defer profileSession.Stop()`. The callers keep the session
// in a local, so it stops when they return, and also when they panic,
// as a Go defer does.

// Go: os.Create (O_RDWR|O_CREATE|O_TRUNC, 0o666), then `panic(err)`.

// Go: pprof/pprof.go:69 CPUProfiler
// CPUProfiler manages on-demand CPU profiling.
// PORT: the session is `ManuallyDrop` because a Go session that nobody
// stops stays open: a `CPUProfiler` that drops while profiling must not
// stop the profile (`ProfileSession` stops when it drops).


// Go: pprof/pprof.go:124 SaveHeapProfile
// SaveHeapProfile saves a heap profile to the specified directory.

// Go: pprof/pprof.go:146 SaveAllocProfile
// SaveAllocProfile saves an allocation profile to the specified directory.

// Go: pprof/pprof.go:167 RunGC
// RunGC triggers garbage collection.
// PORT: Go `runtime.GC()`. Rust has no garbage collector, so there is
// nothing to do.
pub fn run_gc() {}

// Go: filepath.Join(profileDir, fmt.Sprintf("%d-%d-<kind>.pb.gz", os.Getpid(),
// time.Now().UnixMilli())). `profileDir` is not empty: `MkdirAll("")` fails
// first.

// Go: fmt.Errorf("<text>: %w", err)
fn wrap(text: &str, err: GoError) -> GoError {
    errors::errorf(format!("{text}: {err}"), vec![err])
}

// PORT: Go `sync.Mutex.Lock`. A panic while the lock is held does not
// poison a Go mutex.

// ---------------------------------------------------------------------
// Go standard library: os (go1.27.1)
// ---------------------------------------------------------------------

// Go: os/path.go:19 MkdirAll, as a Go error (osvfs `os_mkdir_all`).

// Go: os/file.go:399 Create, which returns `&PathError{Op: "open", ...}`.

// Go: io/fs.PathError.Error, "op path: err", with the Go text of the errno.

// ---------------------------------------------------------------------
// Go standard library: runtime/pprof (go1.27.1)
// ---------------------------------------------------------------------

// Go: runtime/pprof/pprof.go:868 cpu (the `profiling` field)
// PORT: Go guards it with a mutex. The atomic swap does the same job.
static CPU_PROFILING: AtomicBool = AtomicBool::new(false);

// Go: runtime/mprof.go:870 MemProfileRate. tsgo does not change it.
const MEM_PROFILE_RATE: i64 = 512 * 1024;

// Go: runtime/pprof/pprof.go:885 StartCPUProfile
// PORT: Go starts the runtime sampler with `runtime.SetCPUProfileRate(hz)`
// and runs `profileWriter(w)` in a goroutine, which owns the profile
// builder and writes it to `w` when profiling stops. The port returns
// that builder, and `stop_cpu_profile` takes it back with `w`.

// Go: runtime/pprof/pprof.go:947 StopCPUProfile
// StopCPUProfile stops the current CPU profile, if any.
// PORT: Go `<-cpu.done` waits for `profileWriter`, which reads the end
// of the profile, calls `b.build()` and ignores its error.

// Go: runtime/pprof/pprof.go:618 writeHeap
// PORT: Go `pprof.Lookup("heap").WriteTo(w, 0)` calls it with debug 0, so
// `writeHeapInternal` calls `writeHeapProto` with no default sample type
// (see `write_alloc`).

// Go: runtime/pprof/pprof.go:624 writeAlloc
// PORT: Go `pprof.Lookup("allocs").WriteTo(w, 0)` calls it with debug 0,
// so `writeHeapInternal` (pprof.go:628) calls `writeHeapProto` with the
// runtime memory profile records and `runtime.MemProfileRate`. The port
// has no records (see the module comment).

// Go: runtime/pprof/protomem.go:16 writeHeapProto
// writeHeapProto writes the current heap profile in protobuf format to w.
// PORT: the record list `p` is always empty, so the loop over it is not
// ported.

// Go: runtime/pprof/proto.go:70 (the tags that a profile with no samples
// writes)
// message Profile
const TAG_PROFILE_SAMPLE_TYPE: u64 = 1; // repeated ValueType
const TAG_PROFILE_MAPPING: u64 = 3; // repeated Mapping
const TAG_PROFILE_STRING_TABLE: u64 = 6; // repeated string
const TAG_PROFILE_TIME_NANOS: u64 = 9; // int64
const TAG_PROFILE_DURATION_NANOS: u64 = 10; // int64
const TAG_PROFILE_PERIOD_TYPE: u64 = 11; // ValueType (really optional string???)
const TAG_PROFILE_PERIOD: u64 = 12; // int64
const TAG_PROFILE_DEFAULT_SAMPLE_TYPE: u64 = 14; // int64
// message ValueType
const TAG_VALUE_TYPE_TYPE: u64 = 1; // int64 (string table index)
const TAG_VALUE_TYPE_UNIT: u64 = 2; // int64 (string table index)
// message Mapping
const TAG_MAPPING_ID: u64 = 1; // uint64
const TAG_MAPPING_START: u64 = 2; // uint64
const TAG_MAPPING_LIMIT: u64 = 3; // uint64
const TAG_MAPPING_OFFSET: u64 = 4; // uint64
const TAG_MAPPING_FILENAME: u64 = 5; // int64 (string table index)
const TAG_MAPPING_BUILD_ID: u64 = 6; // int64 (string table index)

// Go: runtime/pprof/proto.go:27 profileBuilder
// A profileBuilder writes a profile incrementally from a
// stream of profile samples delivered by the runtime.
// PORT: only the fields that a profile with no samples uses. Go makes the
// gzip writer `zw` in `newProfileBuilder`. It writes nothing before its
// first `Write`, so `build` makes it. Go `time.Time` holds a wall and a
// monotonic reading: `start` is the wall reading for `UnixNano`, and
// `start_mono` the monotonic one for `Sub`. Go strings hold any bytes, so
// the string table holds bytes.
struct ProfileBuilder {
    start: SystemTime,
    start_mono: Instant,
    have_period: bool,
    period: i64,

    // encoding state
    pb: Protobuf,
    strings: Vec<Vec<u8>>,
    string_map: FxHashMap<Vec<u8>, i64>,
    mem: Vec<MemMap>,
}

// Go: runtime/pprof/proto.go:46 memMap
// PORT: `funcs` and `fake` serve locations, which a profile with no
// samples does not have.
struct MemMap {
    start: u64,       // Address at which the binary (or DLL) is loaded into memory.
    end: u64,         // The limit of the address range occupied by this mapping.
    offset: u64,      // Offset in the binary that corresponds to the first mapped address.
    file: Vec<u8>,    // The object this entry is loaded from.
    build_id: String, // A string that uniquely identifies a particular program version with high probability.
}

impl ProfileBuilder {
    // Go: runtime/pprof/proto.go:259 newProfileBuilder

    // Go: runtime/pprof/proto.go:133 stringIndex
    // stringIndex adds s to the string table if not already present
    // and returns the index of s in the string table.
    fn string_index(&mut self, s: &[u8]) -> i64 {
        if let Some(&id) = self.string_map.get(s) {
            return id;
        }
        let id = self.strings.len() as i64;
        self.strings.push(s.to_vec());
        self.string_map.insert(s.to_vec(), id);
        id
    }

    // Go: runtime/pprof/proto.go:152 pbValueType
    // pbValueType encodes a ValueType message to b.pb.

    // Go: runtime/pprof/proto.go:189 pbMapping
    // pbMapping encodes a Mapping message to b.pb.
    // PORT: Go writes `HasFunctions` only for a mapping whose locations
    // were all symbolized. A profile with no samples has no locations, so
    // `hasFuncs` is always false and not ported.

    // Go: runtime/pprof/proto.go:348 build
    // build completes and returns the constructed profile.
    // PORT: Go writes a `Sample` message for each profiled stack between
    // the header and the mappings. The port has no samples.

    // Go: runtime/pprof/proto_other.go:17 readMapping
    // readMapping reads /proc/self/maps and writes mappings to b.pb.
    // It saves the address ranges of the mappings in b.mem for use
    // when emitting locations.

    // Go: runtime/pprof/proto.go:760 addMappingEntry
    // PORT: Go `addMapping` (proto.go:756) calls it with `fake` false.
    // The `fake` field is not ported (see `MemMap`).
    fn add_mapping_entry(&mut self, lo: u64, hi: u64, offset: u64, file: &[u8], build_id: String) {
        self.mem.push(MemMap {
            start: lo,
            end: hi,
            offset,
            file: file.to_vec(),
            build_id,
        });
    }
}

// Go: runtime/pprof/proto.go:665 parseProcSelfMaps

// Go: bytes.Cut(s, []byte{sep}). `after` is empty when `sep` is missing
// (Go returns nil, which the callers treat the same way).
fn cut(s: &[u8], sep: u8) -> (&[u8], &[u8]) {
    match crate::memchr::memchr(sep, s) {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, &[]),
    }
}

// Go: bytes.TrimLeft(s, " ")
fn trim_left_spaces(s: &[u8]) -> &[u8] {
    let n = s.iter().take_while(|&&c| c == b' ').count();
    &s[n..]
}

// Go: strconv.ParseUint(s, 16, 64). Rust `from_str_radix` also takes a
// leading `+`, which Go rejects.

// Go: time.Time.UnixNano
fn unix_nano(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_nanos() as i64,
        Err(err) => -(err.duration().as_nanos() as i64),
    }
}

// Go: time.Time.UnixMilli
fn unix_milli(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as i64,
        Err(err) => -(err.duration().as_millis() as i64),
    }
}

// Go: runtime/pprof/elf.go:21 elfBuildID
// elfBuildID returns the GNU build ID of the named ELF binary,
// without introducing a dependency on debug/elf and its dependencies.
// PORT: Go returns an error that its only caller ignores. `None` is
// that error.

// Go: (*os.File).ReadAt, which fails on a negative offset and on a short
// read.

// PORT: off unix, a seek and a read (the only caller reads one file on one
// thread).
#[cfg(not(unix))]
fn read_at(mut f: &File, buf: &mut [u8], off: i64) -> Option<()> {
    use std::io::{Read, Seek, SeekFrom};
    f.seek(SeekFrom::Start(u64::try_from(off).ok()?)).ok()?;
    f.read_exact(buf).ok()
}

// Go: runtime/pprof/protobuf.go:8 protobuf
// A protobuf is a simple protocol buffer encoder.
// PORT: Go `tmp` holds the length bytes that `endMessage` moves, and
// `nest` serves `profileBuilder.flush`. `end_message` rotates the slice
// instead, and `flush` serves samples and locations, which the port does
// not write. So neither field is ported.
#[derive(Default)]
struct Protobuf {
    data: Vec<u8>,
}

impl Protobuf {
    // Go: runtime/pprof/protobuf.go:14 varint
    fn varint(&mut self, mut x: u64) {
        while x >= 128 {
            self.data.push(x as u8 | 0x80);
            x >>= 7;
        }
        self.data.push(x as u8);
    }

    // Go: runtime/pprof/protobuf.go:22 length
    fn length(&mut self, tag: u64, len: usize) {
        self.varint(tag << 3 | 2);
        self.varint(len as u64);
    }

    // Go: runtime/pprof/protobuf.go:27 uint64
    fn uint64(&mut self, tag: u64, x: u64) {
        // append varint to b.data
        self.varint(tag << 3);
        self.varint(x);
    }

    // Go: runtime/pprof/protobuf.go:53 uint64Opt
    fn uint64_opt(&mut self, tag: u64, x: u64) {
        if x == 0 {
            return;
        }
        self.uint64(tag, x);
    }

    // Go: runtime/pprof/protobuf.go:60 int64
    fn int64(&mut self, tag: u64, x: i64) {
        self.uint64(tag, x as u64);
    }

    // Go: runtime/pprof/protobuf.go:65 int64Opt
    fn int64_opt(&mut self, tag: u64, x: i64) {
        if x == 0 {
            return;
        }
        self.int64(tag, x);
    }

    // Go: runtime/pprof/protobuf.go:92 string
    fn string(&mut self, tag: u64, x: &[u8]) {
        self.length(tag, x.len());
        self.data.extend_from_slice(x);
    }

    // Go: runtime/pprof/protobuf.go:97 strings
    fn strings(&mut self, tag: u64, x: &[Vec<u8>]) {
        for s in x {
            self.string(tag, s);
        }
    }

    // Go: runtime/pprof/protobuf.go:127 startMessage
    fn start_message(&mut self) -> usize {
        self.data.len()
    }

    // Go: runtime/pprof/protobuf.go:132 endMessage
    // PORT: Go appends the length, then moves its bytes in front of the
    // message with `tmp`. `rotate_right` moves them the same way.
}
