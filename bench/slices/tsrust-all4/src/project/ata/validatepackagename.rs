//! Go `internal/project/ata/validatepackagename.go`.

use crate::project::ata::prelude::*;

use crate::gostd;

// Go: project/ata/validatepackagename.go:10 NameValidationResult
// PORT: Go `type NameValidationResult int` with `iota` consts. The consts
// keep their Go names as package-level consts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NameValidationResult(pub i32);

// Go: project/ata/validatepackagename.go:12 NameOk .. NameContainsNonURISafeCharacters
pub const NAME_OK: NameValidationResult = NameValidationResult(0);
pub const EMPTY_NAME: NameValidationResult = NameValidationResult(1);
pub const NAME_TOO_LONG: NameValidationResult = NameValidationResult(2);
pub const NAME_STARTS_WITH_DOT: NameValidationResult = NameValidationResult(3);
pub const NAME_STARTS_WITH_UNDERSCORE: NameValidationResult = NameValidationResult(4);
pub const NAME_CONTAINS_NON_URI_SAFE_CHARACTERS: NameValidationResult = NameValidationResult(5);

// Go: project/ata/validatepackagename.go:21 maxPackageNameLength
// PORT: Go untyped const compared with `len`, so `usize`.
pub const MAX_PACKAGE_NAME_LENGTH: usize = 214;

// Go: project/ata/validatepackagename.go:28 ValidatePackageName
// Validates package name using rules defined at https://docs.npmjs.com/files/package.json
//
// @internal

// Go: project/ata/validatepackagename.go:32 validatePackageNameWorker

// Go: project/ata/validatepackagename.go:72 renderPackageNameValidationFailure
// @internal
