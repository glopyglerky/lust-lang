// UTF-8 decoding from bytes in body checking (W-STD-A phase 3):
// `String::from_utf8`, `str::from_utf8` (also `std::str::from_utf8`),
// `String::from_utf8_lossy` (a `Cow<str>`), and the methods of their
// errors. `FromUtf8Error` and `Utf8Error` are
// synthetic std struct Adts (collect.rs, `generics::std_adts`), so
// their derived Debug, Clone, and Eq are the plain struct machinery;
// Display and the methods are builtins (cgen.rs).
use super::*;

impl<'p> Ctx<'p> {
    /// std's `FromUtf8Error` (`from`) or `Utf8Error` type: always
    /// registered (check/collect.rs), even where a user type takes the
    /// flat name.
    fn utf8_error_ty(&mut self, from: bool) -> Ty {
        let def = if from { generics::from_utf8_error_def() } else { generics::utf8_error_def() };
        self.tys.intern(TyKind::Adt(def, EMPTY_ARGS))
    }

    /// Parameter and return types of `String::from_utf8`,
    /// `str::from_utf8`, and `String::from_utf8_lossy`.
    pub(super) fn utf8_call_sig(&mut self, b: Builtin) -> (Vec<Ty>, Ty) {
        let bytes = int(IntTy::U8);
        if b == Builtin::StringFromUtf8 {
            let v = self.tys.mk_vec(bytes);
            let e = self.utf8_error_ty(true);
            (vec![v], self.tys.mk_result(STRING, e))
        } else if b == Builtin::StringFromUtf16Lossy {
            let sl = self.tys.intern(TyKind::Slice(int(IntTy::U16)));
            let r = self.tys.mk_ref(false, sl);
            (vec![r], STRING)
        } else if b == Builtin::StringFromUtf8Lossy {
            let sl = self.tys.intern(TyKind::Slice(bytes));
            let r = self.tys.mk_ref(false, sl);
            let cow = self.tys.mk_adt(generics::cow_def(), &[GenericArg::Ty(STR)]);
            (vec![r], cow)
        } else {
            let sl = self.tys.intern(TyKind::Slice(bytes));
            let r = self.tys.mk_ref(false, sl);
            let s = self.tys.mk_ref(false, STR);
            let e = self.utf8_error_ty(false);
            (vec![r], self.tys.mk_result(s, e))
        }
    }

    /// The UTF-8 decoding fns: `String::from_utf8` (`string`) or the
    /// `str` one. None when `item` is not one lust knows; the ones it
    /// lacks reject here.
    pub(super) fn utf8_assoc(&mut self, string: bool, item: Sym, span: Span) -> Option<Option<Res>> {
        let n = self.name(item).to_string();
        match (string, n.as_str()) {
            (true, "from_utf8") => Some(Some(Res::Builtin(Builtin::StringFromUtf8))),
            (true, "from_utf16_lossy") => Some(Some(Res::Builtin(Builtin::StringFromUtf16Lossy))),
            (true, "from_utf8_lossy") => Some(Some(Res::Builtin(Builtin::StringFromUtf8Lossy))),
            (false, "from_utf8") => Some(Some(Res::Builtin(Builtin::StrFromUtf8))),
            (true, "from_utf8_unchecked" | "from_utf16")
            | (false, "from_utf8_mut" | "from_utf8_unchecked" | "from_utf8_unchecked_mut") => {
                let t = if string { "String" } else { "str" };
                self.err("L0640", span, format!("`{t}::{n}` is not implemented yet"), "Use `from_utf8` and handle its error.");
                Some(None)
            }
            _ => None,
        }
    }

    /// Methods on `FromUtf8Error` and `Utf8Error` receivers (after
    /// autoderef). None when the receiver is neither, or the name is
    /// not one of these (a user trait may still provide it).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn utf8_method(&mut self, e: ExprId, recv: ExprId, base: Ty, derefs: u8, name: Sym, span: Span, args: &[ExprId]) -> Option<Ty> {
        use Builtin as B;
        // `Utf8Chunk::valid` and `invalid` (tf-slice): the two
        // fields of the chunk an `utf8_chunks` step yields.
        if let Some((crate::ty::iter::IterKind::Utf8Chunk, _)) = crate::ty::iter::iter_parts(&self.tys, base) {
            let (b, ret) = match self.name(name) {
                "valid" => (B::Utf8ChunkValid, REF_STR),
                "invalid" => {
                    let sl = self.tys.intern(TyKind::Slice(int(IntTy::U8)));
                    (B::Utf8ChunkInvalid, self.tys.mk_ref(false, sl))
                }
                _ => return None,
            };
            self.check_packed_borrow(recv, span);
            self.set_res(e, Res::BuiltinMethod { b, derefs, autoref: Autoref::Ref });
            self.check_args(&[], args, span);
            return Some(ret);
        }
        let def = self.tys.adt_parts(base).map(|(d, _)| d)?;
        let n = self.name(name).to_string();
        let (b, autoref, ret) = if n == "to_string" && (def == generics::from_utf8_error_def() || def == generics::utf8_error_def()) {
            // Their `Display` (cgen/fmt.rs `display_glue_body`).
            (B::ToString, Autoref::Ref, STRING)
        } else if def == generics::from_utf8_error_def() {
            match n.as_str() {
                "utf8_error" => (B::FromUtf8ErrorUtf8Error, Autoref::Ref, self.utf8_error_ty(false)),
                "as_bytes" => {
                    let sl = self.tys.intern(TyKind::Slice(int(IntTy::U8)));
                    (B::FromUtf8ErrorAsBytes, Autoref::Ref, self.tys.mk_ref(false, sl))
                }
                "into_bytes" => (B::FromUtf8ErrorIntoBytes, Autoref::None, self.tys.mk_vec(int(IntTy::U8))),
                _ => return None,
            }
        } else if def == generics::parse_int_error_def() && n == "kind" {
            // `ParseIntError::kind(&self) -> &IntErrorKind` (tf-str).
            let k = generics::int_error_kind_ty(&mut self.tys);
            (B::ParseIntErrorKind, Autoref::Ref, self.tys.mk_ref(false, k))
        } else if def == generics::utf8_error_def() {
            match n.as_str() {
                "valid_up_to" => (B::Utf8ErrorValidUpTo, Autoref::Ref, USIZE),
                "error_len" => (B::Utf8ErrorErrorLen, Autoref::Ref, self.tys.mk_option(USIZE)),
                _ => return None,
            }
        } else {
            return None;
        };
        if autoref != Autoref::None {
            self.check_packed_borrow(recv, span);
        }
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref });
        self.check_args(&[], args, span);
        Some(ret)
    }
}
