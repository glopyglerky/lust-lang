//! Port of execute/incremental/buildInfo.go: the `.tsbuildinfo` JSON shapes
//! and their custom JSON (un)marshalers.
//!
//! PORT: Go uses JSON v2 reflection plus `MarshalJSON`/`UnmarshalJSON`
//! methods. Here every type implements `MarshalerTo` and `UnmarshalerFrom`
//! from `frontend/json.rs` by hand, in Go field order, so the output bytes
//! match Go `json.Marshal`:
//! - `omitzero` struct fields are skipped when zero. A Go `[]T` field with
//!   `omitzero` is `Option<Vec<T>>`, because Go writes an empty non-nil
//!   slice as `[]` and omits only a nil slice.
//! - Go `[]*T` elements that are JSON `null` become nil pointers in Go. The
//!   Rust elements are values, so a `null` element is an unmarshal error
//!   (the reader then returns no buildinfo). Go writes such an element only
//!   in `emitDiagnosticsPerFile` (`BuildInfoDiagnosticsOfFilePtr`).
//! - Go `int` is `i32` (PORTING.md). Integers are read like the v2 int
//!   arshaler: digits only, no fraction or exponent.

use super::hash::*;
use crate::contentmapper;
use crate::frontend::prelude::*;
use crate::gostd::GoError;
// `vfs::FileInfo` is also in the frontend prelude.
use super::hash::FileInfo;

// ---------------------------------------------------------------------------
// JSON helpers (Go JSON v2 default arshalers for the kinds used here).
// ---------------------------------------------------------------------------

fn json_error(message: impl Into<String>) -> JsonError {
    JsonError {
        message: message.into(),
    }
}

// Go v2 int arshaler: null gives 0, a number must be an integer in range,
// any other kind is an error after the value is read.

// Go v2 `jsonwire.ParseUint` with a sign: digits only, range of int64.




// Go v2 slice arshaler: null gives a nil slice (`None`), an array gives a
// non-nil slice, any other kind is an error.

// Unmarshal one element of a Go `[]*T` slice.

// Go v2 struct arshaler over a JSON object. `field` returns false for an
// unknown member name, which is skipped (v2 does not reject unknown
// members by default). Null leaves the zero value.

// Go `json.Unmarshal(data, &v)` for the nested calls inside the Go
// `UnmarshalJSON` methods.

// Go `json.Unmarshal` into a value read by a closure.

// Go v2 `[N]int` arshaler: exactly N elements.

// Go v2 `any` arshaler: null, bool, float64, string, []any and
// map[string]any.
// PORT: Go `map[string]any` does not keep member order; the Rust map keeps
// the input order.

// Go `collections.OrderedMap[string, any].UnmarshalJSONFrom`. A pointer
// field that is JSON null stays nil.

fn marshal_int(enc: &mut String, v: i64) {
    enc.push_str(&v.to_string());
}

// Go v2 float64 marshaler (`jsonwire.AppendFloat`): ECMA-262 number
// formatting, except that -0 is "-0".

// Go v2 marshaler for the `any` values stored in `BuildInfo.Options`: the
// values read back from JSON, and the typed `core.CompilerOptions` field
// values that `snapshotToBuildInfo` stores.

// Writes the members of a Go struct in field order.
struct ObjectWriter<'a> {
    enc: &'a mut String,
    first: bool,
}

impl<'a> ObjectWriter<'a> {
    fn new(enc: &'a mut String) -> ObjectWriter<'a> {
        enc.push('{');
        ObjectWriter { enc, first: true }
    }

    fn name(&mut self, name: &str) -> &mut String {
        if !self.first {
            self.enc.push(',');
        }
        self.first = false;
        self.enc.push('"');
        self.enc.push_str(name);
        self.enc.push_str("\":");
        self.enc
    }

    fn bool_omitzero(&mut self, name: &str, v: bool) {
        if v {
            self.name(name).push_str("true");
        }
    }

    fn int_omitzero(&mut self, name: &str, v: i64) {
        if v != 0 {
            marshal_int(self.name(name), v);
        }
    }


    fn slice_omitzero<T: MarshalerTo>(
        &mut self,
        name: &str,
        v: Option<&Vec<T>>,
    ) -> Result<(), JsonError> {
        if let Some(v) = v {
            v.marshal_json_to(self.name(name))?;
        }
        Ok(())
    }

    fn end(self) {
        self.enc.push('}');
    }
}

// ---------------------------------------------------------------------------
// buildInfo.go
// ---------------------------------------------------------------------------

// Go: incremental/buildInfo.go:16 BuildInfoFileId
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BuildInfoFileId(pub i32);

// Go: incremental/buildInfo.go:17 BuildInfoFileIdListId
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BuildInfoFileIdListId(pub i32);


impl MarshalerTo for BuildInfoFileId {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        marshal_int(enc, i64::from(self.0));
        Ok(())
    }
}

// Go: incremental/buildInfo.go:21 BuildInfoRoot
// buildInfoRoot is
// - for incremental program buildinfo
//   - start and end of FileId for consecutive fileIds to be included as root
//   - start - single fileId that is root
//
// - for non incremental program buildinfo
//   - string that is the root file name
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoRoot {
    pub start: BuildInfoFileId,
    pub end: BuildInfoFileId,
    pub non_incremental: String, // Root of a non incremental program
}



// Go: incremental/buildInfo.go:73 buildInfoFileInfoNoSignature
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoFileInfoNoSignature {
    pub version: String,
    pub no_signature: bool,
    pub affects_global_scope: bool,
    pub implied_node_format: ResolutionMode,
}



// Go: incremental/buildInfo.go:83 buildInfoFileInfoWithSignature
//
//	 Signature is
//		 - undefined if FileInfo.version === FileInfo.signature
//		 - string actual signature
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoFileInfoWithSignature {
    pub version: String,
    pub signature: String,
    pub affects_global_scope: bool,
    pub implied_node_format: ResolutionMode,
}



/// PORT: not in Go (perf). The members of both object forms of a
/// `BuildInfoFileInfo`, decoded in one pass. When this decode succeeds,
/// the Go decodes of `buildInfoFileInfoNoSignature` and
/// `buildInfoFileInfoWithSignature` succeed too: each reads a subset of
/// these members with the same decoders and skips the rest, and the value
/// is valid JSON with no duplicate names. So the result is the one of
/// Go's `UnmarshalJSON` for an object.
#[derive(Default)]
struct BuildInfoFileInfoMembers {
    version: String,
    signature: String,
    no_signature: bool,
    affects_global_scope: bool,
    implied_node_format: ResolutionMode,
}

impl BuildInfoFileInfoMembers {
}


// Go: incremental/buildInfo.go:90 BuildInfoFileInfo
// PORT: Go unexported fields are plain `pub` fields. Go nil pointers are
// `None`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoFileInfo {
    pub signature: String,
    pub no_signature: Option<BuildInfoFileInfoNoSignature>,
    pub file_info: Option<BuildInfoFileInfoWithSignature>,
}

// Go: incremental/buildInfo.go:96 newBuildInfoFileInfo
#[must_use]
pub fn new_build_info_file_info(file_info: &FileInfo) -> BuildInfoFileInfo {
    if file_info.version == file_info.signature {
        if !file_info.affects_global_scope
            && file_info.implied_node_format == RESOLUTION_MODE_COMMON_JS
        {
            return BuildInfoFileInfo {
                signature: file_info.signature.clone(),
                ..Default::default()
            };
        }
    } else if file_info.signature.is_empty() {
        return BuildInfoFileInfo {
            no_signature: Some(BuildInfoFileInfoNoSignature {
                version: file_info.version.clone(),
                no_signature: true,
                affects_global_scope: file_info.affects_global_scope,
                implied_node_format: file_info.implied_node_format,
            }),
            ..Default::default()
        };
    }
    BuildInfoFileInfo {
        file_info: Some(BuildInfoFileInfoWithSignature {
            version: file_info.version.clone(),
            signature: if file_info.signature == file_info.version {
                String::new()
            } else {
                file_info.signature.clone()
            },
            affects_global_scope: file_info.affects_global_scope,
            implied_node_format: file_info.implied_node_format,
        }),
        ..Default::default()
    }
}

impl BuildInfoFileInfo {
    // Go: incremental/buildInfo.go:116 GetFileInfo
    // PORT: the Go nil receiver check is the caller's `Option`.
    #[must_use]
    pub fn get_file_info(&self) -> FileInfo {
        if !self.signature.is_empty() {
            return FileInfo {
                version: self.signature.clone(),
                signature: self.signature.clone(),
                implied_node_format: RESOLUTION_MODE_COMMON_JS,
                ..Default::default()
            };
        }
        if let Some(no_signature) = &self.no_signature {
            return FileInfo {
                version: no_signature.version.clone(),
                affects_global_scope: no_signature.affects_global_scope,
                implied_node_format: no_signature.implied_node_format,
                ..Default::default()
            };
        }
        let file_info = self
            .file_info
            .as_ref()
            .expect("BuildInfoFileInfo has a fileInfo");
        FileInfo {
            version: file_info.version.clone(),
            signature: if file_info.signature.is_empty() {
                file_info.version.clone()
            } else {
                file_info.signature.clone()
            },
            affects_global_scope: file_info.affects_global_scope,
            implied_node_format: file_info.implied_node_format,
        }
    }

    // Go: incremental/buildInfo.go:142 HasSignature
    #[must_use]
    pub fn has_signature(&self) -> bool {
        !self.signature.is_empty()
    }
}



// Go: incremental/buildInfo.go:176 BuildInfoReferenceMapEntry
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoReferenceMapEntry {
    pub file_id: BuildInfoFileId,
    pub file_id_list_id: BuildInfoFileIdListId,
}

impl MarshalerTo for BuildInfoReferenceMapEntry {
    // Go: incremental/buildInfo.go:181 MarshalJSON
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        enc.push('[');
        marshal_int(enc, i64::from(self.file_id.0));
        enc.push(',');
        marshal_int(enc, i64::from(self.file_id_list_id.0));
        enc.push(']');
        Ok(())
    }
}


// Go: incremental/buildInfo.go:197 BuildInfoDiagnostic
// PORT: Go `diagnostics.Category` (int32) is stored as `i32`, so the zero
// value (`CategoryWarning`) is omitted like in Go. Go `diagnostics.Key` is
// `String`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoDiagnostic {
    // BuildInfoFileId if it is for a File thats other than its stored for
    pub file: BuildInfoFileId,
    pub no_file: bool,
    pub pos: i32,
    pub end: i32,
    pub code: i32,
    pub category: i32,
    pub source: String,
    pub message_text: String,
    pub message_key: String,
    pub message_args: Option<Vec<String>>,
    pub message_chain: Option<Vec<BuildInfoDiagnostic>>,
    pub related_information: Option<Vec<BuildInfoDiagnostic>>,
    pub reports_unnecessary: bool,
    pub reports_deprecated: bool,
    pub skipped_on_no_emit: bool,
    pub repopulate_info: Option<BuildInfoRepopulateInfo>,
}



// Go: incremental/buildInfo.go:214 BuildInfoRepopulateInfo
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoRepopulateInfo {
    pub kind: RepopulateDiagnosticKind,
    pub module_reference: String,
    pub mode: ResolutionMode,
    pub package_name: String,
}



// Go: incremental/buildInfo.go:221 BuildInfoDiagnosticsOfFile
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoDiagnosticsOfFile {
    pub file_id: BuildInfoFileId,
    pub diagnostics: Vec<BuildInfoDiagnostic>,
}



/// Go `*BuildInfoDiagnosticsOfFile`, an element of
/// `BuildInfo.EmitDiagnosticsPerFile`. `None` is Go nil.
// PORT: `setEmitDiagnostics` keeps each `toBuildInfoDiagnosticsOfFile`
// result, and that is nil for a file whose cached lists are both empty. Go
// marshals the nil element as `null`. Field reads go through `Deref`, which
// panics on nil like a Go nil pointer dereference.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoDiagnosticsOfFilePtr(pub Option<BuildInfoDiagnosticsOfFile>);

impl std::ops::Deref for BuildInfoDiagnosticsOfFilePtr {
    type Target = BuildInfoDiagnosticsOfFile;

    fn deref(&self) -> &BuildInfoDiagnosticsOfFile {
        self.0
            .as_ref()
            .expect("invalid memory address or nil pointer dereference")
    }
}



// Go: incremental/buildInfo.go:233 UnmarshalJSON (body)

// Go: incremental/buildInfo.go:258 BuildInfoSemanticDiagnostic
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoSemanticDiagnostic {
    pub file_id: BuildInfoFileId, // File is not in changedSet and still doesnt have cached diagnostics
    pub diagnostics: Option<BuildInfoDiagnosticsOfFile>, // Diagnostics for file
}



// Go: incremental/buildInfo.go:289 BuildInfoFilePendingEmit
// fileId if pending emit is same as what compilerOptions suggest
// [fileId] if pending emit is only dts file emit
// [fileId, emitKind] if any other type emit is pending
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoFilePendingEmit {
    pub file_id: BuildInfoFileId,
    pub emit_kind: FileEmitKind,
}

impl MarshalerTo for BuildInfoFilePendingEmit {
    // Go: incremental/buildInfo.go:294 MarshalJSON
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        if self.emit_kind.is_empty() {
            return self.file_id.marshal_json_to(enc);
        }
        if self.emit_kind == FileEmitKind::DTS {
            let file_list_ids = vec![self.file_id];
            return file_list_ids.marshal_json_to(enc);
        }
        enc.push('[');
        marshal_int(enc, i64::from(self.file_id.0));
        enc.push(',');
        marshal_int(enc, i64::from(self.emit_kind.0));
        enc.push(']');
        Ok(())
    }
}


// Go: incremental/buildInfo.go:338 BuildInfoEmitSignature
// [fileId, signature] if different from file's signature
// fileId if file wasnt emitted
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoEmitSignature {
    pub file_id: BuildInfoFileId,
    pub signature: String, // Signature if it is different from file's Signature
    pub differs_only_in_dts_map: bool, // true if signature is different only in dtsMap value
    pub differs_in_options: bool, // true if signature is different in options used to emit file
}

impl BuildInfoEmitSignature {
    // Go: incremental/buildInfo.go:345 noEmitSignature
    #[must_use]
    pub fn no_emit_signature(&self) -> bool {
        self.signature.is_empty() && !self.differs_only_in_dts_map && !self.differs_in_options
    }

    // Go: incremental/buildInfo.go:350 toEmitSignature
    // PORT: Go `collections.SyncMap[tspath.Path, *emitSignature]` is a plain
    // map. Go dereferences a missing entry (nil pointer panic); so does the
    // `expect`.
    #[must_use]
    pub fn to_emit_signature(
        &self,
        path: &Path,
        emit_signatures: &FxHashMap<Path, EmitSignature>,
    ) -> EmitSignature {
        let mut signature = String::new();
        let mut signature_with_different_options: Option<Vec<String>> = None;
        if self.differs_only_in_dts_map {
            let mut list = Vec::with_capacity(1);
            let info = emit_signatures
                .get(path)
                .expect("emit signature for differsOnlyInDtsMap");
            list.push(info.signature.clone());
            signature_with_different_options = Some(list);
        } else if self.differs_in_options {
            let mut list = Vec::with_capacity(1);
            list.push(self.signature.clone());
            signature_with_different_options = Some(list);
        } else {
            signature = self.signature.clone();
        }
        EmitSignature {
            signature,
            signature_with_different_options,
        }
    }
}



// Go: incremental/buildInfo.go:444 BuildInfoResolvedRoot
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildInfoResolvedRoot {
    pub resolved: BuildInfoFileId,
    pub root: BuildInfoFileId,
}

impl MarshalerTo for BuildInfoResolvedRoot {
    // Go: incremental/buildInfo.go:449 MarshalJSON
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        vec![self.resolved, self.root].marshal_json_to(enc)
    }
}


// Go: incremental/buildInfo.go:465 BuildInfo
// PORT: Go `*collections.OrderedMap[string, any]` is
// `Option<IndexMap<String, CompilerOptionsValue>>`. Go `any` values read
// from JSON are `Nil`, `Bool`, `Number`, `String`, `List` or `Map`.



// Go: incremental/buildInfo.go:501 ContentMapperIdentities (tsgo#4712)
// ContentMapperIdentities returns the project's sorted mapper transform identities. A nil project means
// the compiler host has no configured content mappers.
// PORT: Go nil (the slice and the project) is `None`.

// Go: core/version.go Version, with Effect-TS/tsgo patch 021


// Go: incremental/buildInfo.go:498 IsBuildInfoFileNameDefaultLibrary

// Go: incremental/buildInfo.go:552 getNormalizedPaths
// PORT: both lifetimes stay separate (edition 2024 captures both) so the
// public getters can return this opaque type.

// Go: incremental/buildInfo.go:613 BuildInfoRootInfoReader


#[cfg(test)]
mod tests {
    use super::*;

    // Every buildinfo shape in one file: the three root forms, the three
    // file info forms, options with a map and a list, nested diagnostics,
    // the three pending emit forms and the four emit signature forms.
    const SAMPLE: &str = r#"{"version":"FakeTSVersion","errors":true,"checkPending":true,"root":[[2,3],4,"../x.ts"],"fileNames":["lib.d.ts","./a.ts","./b.ts","./c.ts","./d.ts"],"fileInfos":["abc",{"version":"v","signature":"s","affectsGlobalScope":true,"impliedNodeFormat":1},{"version":"v2","noSignature":true,"impliedNodeFormat":99},{"version":"v3"},"e"],"fileIdsList":[[2],[]],"options":{"composite":true,"module":199,"outDir":"./dist","paths":{"a":["b"]},"lib":["lib.es2022.d.ts"]},"referencedMap":[[3,1]],"semanticDiagnosticsPerFile":[1,[2,[{"pos":1,"end":2,"code":2322,"category":1,"messageKey":"k","messageArgs":["a\"\n"],"messageChain":[{"code":1,"messageKey":"m","repopulateInfo":{"kind":1,"moduleReference":"x","mode":99}}],"relatedInformation":[{"file":3,"pos":3,"end":4,"code":2728,"category":3,"messageKey":"r"}]}]]],"emitDiagnosticsPerFile":[[2,[{"end":1,"code":9010,"category":1,"messageKey":"i","skippedOnNoEmit":true}]]],"changeFileSet":[3],"affectedFilesPendingEmit":[2,[3],[4,17]],"latestChangedDtsFile":"./dist/a.d.ts","emitSignatures":[2,[3,"sig"],[4,[]],[5,["s2"]]],"resolvedRoot":[[2,5]],"semanticErrors":true}"#;

    #[test]
    fn round_trips_every_shape() {
        let mut info = BuildInfo::default();
        json_unmarshal(SAMPLE.as_bytes(), &mut info, &[]).unwrap();
        assert_eq!(json_marshal(&info, &[]).unwrap(), SAMPLE);

        let root = info.root.as_ref().unwrap();
        assert_eq!(
            (root[0].start, root[0].end),
            (BuildInfoFileId(2), BuildInfoFileId(3))
        );
        assert_eq!(root[2].non_incremental, "../x.ts");
        assert!(info.file_info(BuildInfoFileId(1)).unwrap().has_signature());
        assert!(
            info.file_info(BuildInfoFileId(3))
                .unwrap()
                .no_signature
                .is_some()
        );
        assert_eq!(
            info.file_info(BuildInfoFileId(4))
                .unwrap()
                .get_file_info()
                .signature,
            "v3"
        );
        let pending = info.affected_files_pending_emit.as_ref().unwrap();
        assert_eq!(pending[1].emit_kind, FileEmitKind::DTS);
        let signatures = info.emit_signatures.as_ref().unwrap();
        assert!(signatures[2].differs_only_in_dts_map);
        assert!(signatures[3].differs_in_options);
    }

    // The one-pass object decode (`BuildInfoFileInfoMembers`) gives what Go's
    // three decodes give, also where it fails and the Go path runs.
    #[test]
    fn file_info_object_decode_matches_go_order() {
        fn go_order(data: &[u8]) -> Option<BuildInfoFileInfo> {
            if let Ok(signature) = unmarshal_bytes::<String>(data) {
                return Some(BuildInfoFileInfo {
                    signature,
                    ..Default::default()
                });
            }
            if let Ok(no_signature) = unmarshal_bytes::<BuildInfoFileInfoNoSignature>(data)
                && no_signature.no_signature
            {
                return Some(BuildInfoFileInfo {
                    no_signature: Some(no_signature),
                    ..Default::default()
                });
            }
            let file_info = unmarshal_bytes::<BuildInfoFileInfoWithSignature>(data).ok()?;
            Some(BuildInfoFileInfo {
                file_info: Some(file_info),
                ..Default::default()
            })
        }
        for data in [
            r#"{"version":"v","signature":"s","affectsGlobalScope":true,"impliedNodeFormat":1}"#,
            r#"{"version":"v","noSignature":true,"impliedNodeFormat":99}"#,
            r#"{"version":"v","noSignature":false}"#,
            r#"{"version":"v","noSignature":true,"signature":"s"}"#,
            r#"{"version":"v","signature":1,"noSignature":true}"#,
            r#"{"version":"v","noSignature":"x","signature":"s"}"#,
            r#"{"version":null,"extra":[1,{"a":2}],"impliedNodeFormat":null}"#,
            r#"{"version":"v","version":"w"}"#,
            r#"{"version":1}"#,
            r#"{}"#,
        ] {
            let mut decoded = BuildInfoFileInfo::default();
            let got = json_unmarshal(data.as_bytes(), &mut decoded, &[])
                .ok()
                .map(|()| decoded);
            assert_eq!(got, go_order(data.as_bytes()), "{data}");
        }
    }

    #[test]
    fn rejects_invalid_shapes() {
        let mut info = BuildInfo::default();
        assert!(json_unmarshal(br#"{"root":[[1,2,3]]}"#, &mut info, &[]).is_err());
        assert!(json_unmarshal(br#"{"root":[1.5]}"#, &mut info, &[]).is_err());
        assert!(json_unmarshal(br#"{"emitSignatures":[[1,[1]]]}"#, &mut info, &[]).is_err());
    }
}
