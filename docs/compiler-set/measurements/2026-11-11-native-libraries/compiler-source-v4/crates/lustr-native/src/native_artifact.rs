//! Independently compiled native libraries. This is Lustr metadata, not rustc
//! metadata. The ar members contain actual relocatable objects and checked
//! export signatures; dependency source is never stored or recompiled.

use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

const AR_MAGIC: &[u8] = b"!<arch>\n";
const META_MAGIC: &[u8] = b"LUSTRMD\x01";
const MAX_ARCHIVE: usize = 256 * 1024 * 1024;
const MAX_METADATA: usize = 16 * 1024 * 1024;
const MAX_OBJECT: usize = 64 * 1024 * 1024;
const MAX_ROWS: usize = 100_000;
const MAX_STRING: usize = 16 * 1024;
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbiType {
    Unit, Bool, Char, I8, I16, I32, I64, I128, Isize,
    U8, U16, U32, U64, U128, Usize, F32, F64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    pub panic_abort: bool,
    pub overflow_checks: bool,
    pub debug_assertions: bool,
    pub debug_info: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Export {
    pub path: Vec<String>,
    pub symbol: String,
    pub params: Vec<AbiType>,
    pub result: AbiType,
    pub is_unsafe: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Location {
    /// The 16-hex-digit function identity, without the `f_` prefix.
    pub symbol: String,
    pub file: String,
    pub line: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeArtifact {
    pub crate_name: String,
    pub disambiguator: String,
    pub target: String,
    pub compiler: String,
    pub runtime: String,
    pub profile: Profile,
    pub exports: Vec<Export>,
    pub locations: Vec<Location>,
    pub objects: Vec<(String, Vec<u8>)>,
}

fn fail(message: impl AsRef<str>) -> String {
    format!("invalid Lustr native library: {}", message.as_ref())
}

fn valid_string(value: &str) -> bool {
    value.len() <= MAX_STRING && !value.chars().any(|c| c == '\0' || c.is_control())
}

fn stable_id(value: &str) -> bool {
    value.len() == 16 && value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn validate_object(name: &str, bytes: &[u8]) -> Result<(), String> {
    if name.is_empty() || name.len() > 15 || !name.ends_with(".o")
        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.')) {
        return Err(fail("object member must be a short, plain .o filename"));
    }
    // Format v1 is explicitly restricted to ELF64 little-endian x86-64
    // relocatables. A renamed executable or source member is not an object.
    if bytes.len() < 64 || bytes.len() > MAX_OBJECT || &bytes[..4] != b"\x7fELF"
        || bytes[4] != 2 || bytes[5] != 1 || bytes[6] != 1
        || u16::from_le_bytes([bytes[16], bytes[17]]) != 1
        || u16::from_le_bytes([bytes[18], bytes[19]]) != 62
        || u32::from_le_bytes(bytes[20..24].try_into().unwrap()) != 1 {
        return Err(fail(format!("{name} is not an x86-64 ELF relocatable object")));
    }
    Ok(())
}

pub fn validate(a: &NativeArtifact) -> Result<(), String> {
    if a.crate_name.is_empty() || !valid_string(&a.crate_name)
        || !a.crate_name.chars().all(|c| c == '_' || c.is_alphanumeric()) {
        return Err(fail("invalid crate name"));
    }
    if a.target != "x86_64-unknown-linux-gnu" {
        return Err(fail("artifact format v1 supports x86_64-unknown-linux-gnu only"));
    }
    for (key, value) in [("disambiguator", &a.disambiguator), ("compiler", &a.compiler), ("runtime", &a.runtime)] {
        if value.is_empty() || !valid_string(value) { return Err(fail(format!("invalid {key}"))); }
    }
    if a.exports.len() > MAX_ROWS || a.locations.len() > MAX_ROWS
        || a.objects.is_empty() || a.objects.len() > MAX_ROWS {
        return Err(fail("invalid member or metadata row count"));
    }
    let mut paths = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    for e in &a.exports {
        if e.path.is_empty() || e.path.len() > 64 || e.path.iter().any(|s|
            s.is_empty() || !valid_string(s) || !s.chars().all(|c| c == '_' || c.is_alphanumeric())) {
            return Err(fail("invalid exported Rust path"));
        }
        if !paths.insert(e.path.clone()) { return Err(fail("duplicate exported path")); }
        if !e.symbol.strip_prefix("f_").is_some_and(stable_id) || !symbols.insert(e.symbol.clone()) {
            return Err(fail("invalid or duplicate exported symbol"));
        }
        if e.params.len() > 1024 { return Err(fail("too many function parameters")); }
    }
    let mut locations = BTreeSet::new();
    for l in &a.locations {
        if !stable_id(&l.symbol) || !locations.insert(l.symbol.clone()) || !valid_string(&l.file) {
            return Err(fail("invalid or duplicate function location"));
        }
    }
    if a.exports.iter().any(|e| !locations.contains(&e.symbol[2..])) {
        return Err(fail("an exported function has no source location record"));
    }
    let mut members = BTreeSet::new();
    let mut total = 0usize;
    for (name, bytes) in &a.objects {
        if !members.insert(name) { return Err(fail("duplicate object member")); }
        validate_object(name, bytes)?;
        total = total.checked_add(bytes.len()).ok_or_else(|| fail("archive size overflow"))?;
        if total > MAX_ARCHIVE { return Err(fail("archive exceeds size limit")); }
    }
    Ok(())
}

// A corruption checksum, not a signature or a trust boundary. Compiler/runtime
// identities and the checked ABI must also match before a caller imports it.
fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x100000001b3))
}

fn put_u32(out: &mut Vec<u8>, value: usize) -> Result<(), String> {
    out.extend_from_slice(&u32::try_from(value).map_err(|_| fail("metadata length overflow"))?.to_le_bytes());
    Ok(())
}

fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), String> {
    put_u32(out, value.len())?;
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn encode_metadata(a: &NativeArtifact) -> Result<Vec<u8>, String> {
    let mut out = META_MAGIC.to_vec();
    for value in [&a.crate_name, &a.disambiguator, &a.target, &a.compiler, &a.runtime] {
        put_string(&mut out, value)?;
    }
    let p = &a.profile;
    out.push(u8::from(p.panic_abort) | (u8::from(p.overflow_checks) << 1)
        | (u8::from(p.debug_assertions) << 2) | (u8::from(p.debug_info) << 3));
    put_u32(&mut out, a.exports.len())?;
    for e in &a.exports {
        put_u32(&mut out, e.path.len())?;
        for part in &e.path { put_string(&mut out, part)?; }
        put_string(&mut out, &e.symbol)?;
        out.push(u8::from(e.is_unsafe));
        put_u32(&mut out, e.params.len())?;
        out.extend(e.params.iter().map(|t| *t as u8));
        out.push(e.result as u8);
    }
    put_u32(&mut out, a.locations.len())?;
    for l in &a.locations {
        put_string(&mut out, &l.symbol)?;
        put_string(&mut out, &l.file)?;
        out.extend_from_slice(&l.line.to_le_bytes());
    }
    put_u32(&mut out, a.objects.len())?;
    for (name, bytes) in &a.objects {
        put_string(&mut out, name)?;
        put_u32(&mut out, bytes.len())?;
        out.extend_from_slice(&checksum(bytes).to_le_bytes());
    }
    out.extend_from_slice(&checksum(&out).to_le_bytes());
    if out.len() > MAX_METADATA { return Err(fail("metadata exceeds size limit")); }
    Ok(out)
}

struct Decoder<'a> { bytes: &'a [u8], at: usize }
impl<'a> Decoder<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).ok_or_else(|| fail("metadata length overflow"))?;
        let bytes = self.bytes.get(self.at..end).ok_or_else(|| fail("truncated metadata"))?;
        self.at = end;
        Ok(bytes)
    }
    fn byte(&mut self) -> Result<u8, String> { Ok(self.take(1)?[0]) }
    fn u32(&mut self) -> Result<u32, String> { Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap())) }
    fn u64(&mut self) -> Result<u64, String> { Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap())) }
    fn count(&mut self, max: usize) -> Result<usize, String> {
        let n = self.u32()? as usize;
        if n > max || n > self.bytes.len() - self.at { return Err(fail("invalid metadata count")); }
        Ok(n)
    }
    fn string(&mut self) -> Result<String, String> {
        let n = self.count(MAX_STRING)?;
        let s = std::str::from_utf8(self.take(n)?).map_err(|_| fail("non-UTF-8 metadata"))?;
        if !valid_string(s) { return Err(fail("invalid metadata string")); }
        Ok(s.to_owned())
    }
    fn ty(&mut self) -> Result<AbiType, String> {
        use AbiType::*;
        const TYPES: &[AbiType] = &[Unit, Bool, Char, I8, I16, I32, I64, I128, Isize,
            U8, U16, U32, U64, U128, Usize, F32, F64];
        TYPES.get(self.byte()? as usize).copied().ok_or_else(|| fail("unknown ABI type"))
    }
}

fn decode_metadata(bytes: &[u8], objects: Vec<(String, Vec<u8>)>) -> Result<NativeArtifact, String> {
    if bytes.len() > MAX_METADATA || bytes.len() < META_MAGIC.len() + 8 || !bytes.starts_with(META_MAGIC) {
        return Err(fail("unsupported metadata version or size"));
    }
    let (data, check) = bytes.split_at(bytes.len() - 8);
    if checksum(data) != u64::from_le_bytes(check.try_into().unwrap()) { return Err(fail("metadata checksum mismatch")); }
    let mut r = Decoder { bytes: data, at: META_MAGIC.len() };
    let crate_name = r.string()?;
    let disambiguator = r.string()?;
    let target = r.string()?;
    let compiler = r.string()?;
    let runtime = r.string()?;
    let flags = r.byte()?;
    if flags & !15 != 0 { return Err(fail("unknown profile flags")); }
    let profile = Profile { panic_abort: flags & 1 != 0, overflow_checks: flags & 2 != 0,
        debug_assertions: flags & 4 != 0, debug_info: flags & 8 != 0 };
    let mut exports = Vec::new();
    for _ in 0..r.count(MAX_ROWS)? {
        let mut path = Vec::new();
        for _ in 0..r.count(64)? { path.push(r.string()?); }
        let symbol = r.string()?;
        let is_unsafe = match r.byte()? { 0 => false, 1 => true, _ => return Err(fail("invalid function safety flag")) };
        let mut params = Vec::new();
        for _ in 0..r.count(1024)? { params.push(r.ty()?); }
        let result = r.ty()?;
        exports.push(Export { path, symbol, params, result, is_unsafe });
    }
    let mut locations = Vec::new();
    for _ in 0..r.count(MAX_ROWS)? {
        locations.push(Location { symbol: r.string()?, file: r.string()?, line: r.u32()? });
    }
    if r.count(MAX_ROWS)? != objects.len() { return Err(fail("object member count mismatch")); }
    for (name, object) in &objects {
        if r.string()? != *name || r.u32()? as usize != object.len() || r.u64()? != checksum(object) {
            return Err(fail(format!("object identity mismatch: {name}")));
        }
    }
    if r.at != data.len() { return Err(fail("unrecognized trailing metadata")); }
    let a = NativeArtifact { crate_name, disambiguator, target, compiler, runtime, profile, exports, locations, objects };
    validate(&a)?;
    Ok(a)
}

fn append_member(out: &mut Vec<u8>, name: &str, data: &[u8]) {
    // Short SysV ar names deliberately avoid GNU/BSD path extensions. Lustr
    // consumes the object members itself; no linker-specific archive index is
    // needed, and ordinary `ar t`/`ar x` can inspect the real object payload.
    let header = format!("{:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n", format!("{name}/"), 0, 0, 0, "100644", data.len());
    debug_assert_eq!(header.len(), 60);
    out.extend_from_slice(header.as_bytes());
    out.extend_from_slice(data);
    if data.len() % 2 != 0 { out.push(b'\n'); }
}

fn encode(a: &NativeArtifact) -> Result<Vec<u8>, String> {
    validate(a)?;
    let metadata = encode_metadata(a)?;
    let mut out = AR_MAGIC.to_vec();
    append_member(&mut out, "lustr.meta", &metadata);
    for (name, object) in &a.objects { append_member(&mut out, name, object); }
    if out.len() > MAX_ARCHIVE { return Err(fail("archive exceeds size limit")); }
    Ok(out)
}

fn decode(bytes: &[u8]) -> Result<NativeArtifact, String> {
    if !bytes.starts_with(AR_MAGIC) || bytes.len() > MAX_ARCHIVE { return Err(fail("not a supported native ar archive")); }
    let mut at = AR_MAGIC.len();
    let mut metadata = None;
    let mut objects = Vec::new();
    while at < bytes.len() {
        let header = bytes.get(at..at + 60).ok_or_else(|| fail("truncated ar header"))?;
        if &header[58..] != b"`\n" { return Err(fail("invalid ar header")); }
        let name = std::str::from_utf8(&header[..16]).map_err(|_| fail("invalid ar member name"))?
            .trim_end().strip_suffix('/').ok_or_else(|| fail("unsupported ar member naming"))?;
        let size_text = std::str::from_utf8(&header[48..58]).map_err(|_| fail("invalid ar member size"))?.trim_end();
        if size_text.is_empty() || !size_text.bytes().all(|b| b.is_ascii_digit()) { return Err(fail("invalid ar member size")); }
        let size: usize = size_text.parse().map_err(|_| fail("ar member size overflow"))?;
        at += 60;
        let end = at.checked_add(size).filter(|&n| n <= bytes.len()).ok_or_else(|| fail("truncated ar member"))?;
        let data = &bytes[at..end];
        at = end;
        if size % 2 != 0 {
            if bytes.get(at) != Some(&b'\n') { return Err(fail("invalid ar padding")); }
            at += 1;
        }
        if name == "lustr.meta" {
            if metadata.replace(data).is_some() || !objects.is_empty() { return Err(fail("metadata must occur once, first")); }
        } else {
            if metadata.is_none() || objects.len() >= MAX_ROWS { return Err(fail("missing metadata or too many objects")); }
            validate_object(name, data)?;
            objects.push((name.to_owned(), data.to_vec()));
        }
    }
    decode_metadata(metadata.ok_or_else(|| fail("missing Lustr metadata"))?, objects)
}

pub fn read(path: &Path) -> Result<NativeArtifact, String> {
    let file = fs::File::open(path).map_err(|e| format!("cannot read native library {}: {e}", path.display()))?;
    let mut bytes = Vec::new();
    file.take((MAX_ARCHIVE + 1) as u64).read_to_end(&mut bytes)
        .map_err(|e| format!("cannot read native library {}: {e}", path.display()))?;
    decode(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn write(path: &Path, artifact: &NativeArtifact) -> Result<(), String> {
    let bytes = encode(artifact)?;
    write_atomic(path, &bytes)
}

/// Cargo's metadata sidecar contains the same checked signature table and
/// object identities as the library. It is never an empty rustc placeholder.
/// Metadata-only compilation/import remains a separate, unsupported operation.
pub fn write_metadata(path: &Path, artifact: &NativeArtifact) -> Result<(), String> {
    validate(artifact)?;
    write_atomic(path, &encode_metadata(artifact)?)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let filename = path.file_name().ok_or_else(|| "native library has no output filename".to_owned())?.to_string_lossy();
    let temp = path.with_file_name(format!(".{filename}.lustr-{}-{}", std::process::id(), TEMP_ID.fetch_add(1, Ordering::Relaxed)));
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)
        .map_err(|e| format!("cannot create native library {}: {e}", path.display()))?;
    let result = file.write_all(bytes).and_then(|_| {
        drop(file);
        fs::rename(&temp, path)
    });
    if let Err(error) = result {
        let _ = fs::remove_file(&temp);
        return Err(format!("cannot write native library {}: {error}", path.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::OnceLock;

    fn artifact() -> NativeArtifact {
        static OBJECT: OnceLock<Vec<u8>> = OnceLock::new();
        let object = OBJECT.get_or_init(|| {
            let root = std::env::temp_dir().join(format!("lustr-artifact-test-{}", std::process::id()));
            fs::create_dir(&root).unwrap();
            let source = root.join("export.c");
            let output = root.join("export.o");
            fs::write(&source, "long f_0123456789abcdef(long x) { return x + 3; }\n").unwrap();
            assert!(Command::new("cc").args(["-c", "-o"]).arg(&output).arg(&source).status().unwrap().success());
            let bytes = fs::read(output).unwrap();
            fs::remove_dir_all(root).unwrap();
            bytes
        }).clone();
        NativeArtifact {
            crate_name: "dependency".into(), disambiguator: "ab12".into(),
            target: "x86_64-unknown-linux-gnu".into(), compiler: "test-compiler".into(), runtime: "test-runtime".into(),
            profile: Profile { panic_abort: false, overflow_checks: true, debug_assertions: true, debug_info: false },
            exports: vec![Export { path: vec!["answer".into()], symbol: "f_0123456789abcdef".into(),
                params: vec![AbiType::I64], result: AbiType::I64, is_unsafe: false }],
            locations: vec![Location { symbol: "0123456789abcdef".into(), file: "dependency/lib.rs".into(), line: 1 }],
            objects: vec![("unit000000.o".into(), object)],
        }
    }

    #[test]
    fn real_object_and_typed_metadata_round_trip() {
        let a = artifact();
        let bytes = encode(&a).unwrap();
        assert_eq!(decode(&bytes).unwrap(), a);
        let root = std::env::temp_dir().join(format!("lustr-archive-test-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let path = root.join("libdependency.rlib");
        write(&path, &a).unwrap();
        assert_eq!(read(&path).unwrap(), a);
        let sidecar = root.join("libdependency.rmeta");
        write_metadata(&sidecar, &a).unwrap();
        let metadata = fs::read(sidecar).unwrap();
        assert_eq!(decode_metadata(&metadata, a.objects.clone()).unwrap(), a);
        let listing = Command::new("ar").arg("t").arg(&path).output().unwrap();
        assert!(listing.status.success());
        assert_eq!(listing.stdout, b"lustr.meta\nunit000000.o\n");
        let extracted = Command::new("ar").arg("p").arg(&path).arg("unit000000.o").output().unwrap();
        assert!(extracted.status.success());
        assert_eq!(extracted.stdout, a.objects[0].1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupt_and_truncated_artifacts_are_rejected_without_panicking() {
        let original = encode(&artifact()).unwrap();
        for end in 0..original.len() { assert!(decode(&original[..end]).is_err(), "accepted prefix {end}"); }
        for offset in [70, 100, original.len() - 8] {
            let mut corrupt = original.clone();
            corrupt[offset] ^= 1;
            assert!(decode(&corrupt).is_err(), "accepted corrupt byte {offset}");
        }
        let mut trailing = original;
        trailing.extend_from_slice(b"unrecognized data");
        assert!(decode(&trailing).is_err());
    }

    #[test]
    fn wrong_targets_duplicate_paths_and_nonobjects_are_rejected() {
        let a = artifact();
        let mut bad = a.clone();
        bad.target = "aarch64-unknown-linux-gnu".into();
        assert!(encode(&bad).is_err());
        let mut bad = a.clone();
        bad.exports.push(bad.exports[0].clone());
        assert!(encode(&bad).is_err());
        let mut bad = a.clone();
        bad.objects[0].1 = b"source is not a native object".to_vec();
        assert!(encode(&bad).is_err());
        let mut bad = a;
        bad.objects[0].0 = "../escape.o".into();
        assert!(encode(&bad).is_err());
    }
}
