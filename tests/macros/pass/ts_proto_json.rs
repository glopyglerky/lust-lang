// macro: proto_json, handle_json, core_int_json
// source: crates/ts_goport/src/api/proto.rs:49
// source: crates/ts_goport/src/api/proto.rs:160
// source: crates/ts_goport/src/api/proto.rs:3601
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
use std::fmt;

// Minimal stand-ins for the api/proto.rs JSON traits and helpers. The
// decoder just replays the field names the test gives it.
#[derive(Debug)]
pub struct JsonError(pub String);
pub struct JsonDecoder<'a> { pub fields: &'a [&'a str], pub uint: u32, pub text: &'a str }
pub trait MarshalerTo { fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError>; }
pub trait UnmarshalerFrom { fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError>; }
pub trait IsZero { fn is_zero(&self) -> bool; }
impl MarshalerTo for u32 { fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> { enc.push_str(&self.to_string()); Ok(()) } }
impl MarshalerTo for String { fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> { enc.push_str(&format!("{self:?}")); Ok(()) } }
impl MarshalerTo for bool { fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> { enc.push_str(if *self { "true" } else { "false" }); Ok(()) } }
impl UnmarshalerFrom for u32 { fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError> { *self = dec.uint; Ok(()) } }
impl UnmarshalerFrom for String { fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError> { *self = dec.text.to_string(); Ok(()) } }
impl UnmarshalerFrom for bool { fn unmarshal_json_from(&mut self, _dec: &mut JsonDecoder<'_>) -> Result<(), JsonError> { *self = true; Ok(()) } }
impl IsZero for u32 { fn is_zero(&self) -> bool { *self == 0 } }
impl IsZero for String { fn is_zero(&self) -> bool { self.is_empty() } }
impl IsZero for bool { fn is_zero(&self) -> bool { !*self } }

pub fn write_object_start(enc: &mut String) { enc.push('{'); }
pub fn write_object_end(enc: &mut String) { enc.push('}'); }
fn sep(enc: &mut String, first: &mut bool, name: &str) {
    if !*first { enc.push(','); }
    *first = false;
    enc.push_str(&format!("{name:?}:"));
}
pub fn marshal_field<T: MarshalerTo>(enc: &mut String, first: &mut bool, name: &str, v: &T) -> Result<(), JsonError> {
    sep(enc, first, name); v.marshal_json_to(enc)
}
pub fn marshal_field_omitzero<T: MarshalerTo + IsZero>(enc: &mut String, first: &mut bool, name: &str, v: &T) -> Result<(), JsonError> {
    if v.is_zero() { return Ok(()); }
    sep(enc, first, name); v.marshal_json_to(enc)
}
pub fn marshal_field_omitempty<T: MarshalerTo + IsZero>(enc: &mut String, first: &mut bool, name: &str, v: &T) -> Result<(), JsonError> {
    if v.is_zero() { return Ok(()); }
    sep(enc, first, name); v.marshal_json_to(enc)
}
pub fn json_unmarshal_decode<T: UnmarshalerFrom>(dec: &mut JsonDecoder<'_>, v: &mut T) -> Result<(), JsonError> {
    v.unmarshal_json_from(dec)
}
pub fn unmarshal_struct_fields(
    dec: &mut JsonDecoder<'_>, _ty: &str,
    mut f: impl FnMut(&str, &mut JsonDecoder<'_>) -> Result<bool, JsonError>,
) -> Result<bool, JsonError> {
    for i in 0..dec.fields.len() {
        let name = dec.fields[i];
        f(name, dec)?;
    }
    Ok(!dec.fields.is_empty())
}
pub fn go_type_name<T>() -> String { std::any::type_name::<T>().rsplit("::").next().unwrap_or("T").to_string() }
pub mod json_ext {
    use super::{JsonDecoder, JsonError};
    pub fn unmarshal_uint_as<T: From<u32>>(dec: &mut JsonDecoder<'_>, _ty: &String) -> Result<T, JsonError> { Ok(T::from(dec.uint)) }
    pub fn unmarshal_string_as(dec: &mut JsonDecoder<'_>, out: &mut String, _ty: &String) -> Result<(), JsonError> { *out = dec.text.to_string(); Ok(()) }
}
pub struct SemanticError { pub go_type: String }
impl SemanticError {
    pub fn of(_e: &JsonError) -> Option<SemanticError> { None }
    pub fn into_json_error(self) -> JsonError { JsonError(self.go_type) }
}

macro_rules! proto_json {
    (marshal $ty:ident { $($field:ident : $name:literal $mode:ident),* $(,)? }) => {
        impl MarshalerTo for $ty {
            fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
                write_object_start(enc);
                let mut first = true;
                $( proto_json!(@field $mode, enc, first, $name, &self.$field); )*
                write_object_end(enc);
                Ok(())
            }
        }
    };
    (both $ty:ident { $($field:ident : $name:literal $mode:ident),* $(,)? }) => {
        proto_json!(marshal $ty { $($field : $name $mode),* });

        impl UnmarshalerFrom for $ty {
            fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError> {
                let is_object = unmarshal_struct_fields(
                    dec,
                    concat!("api.", stringify!($ty)),
                    |name, dec| {
                        match name {
                            $($name => json_unmarshal_decode(dec, &mut self.$field)?,)*
                            _ => return Ok(false),
                        }
                        Ok(true)
                    },
                )?;
                if !is_object {
                    *self = $ty::default();
                }
                Ok(())
            }
        }
    };
    (@field plain, $enc:ident, $first:ident, $name:literal, $value:expr) => {
        marshal_field($enc, &mut $first, $name, $value)?;
    };
    (@field omitzero, $enc:ident, $first:ident, $name:literal, $value:expr) => {
        marshal_field_omitzero($enc, &mut $first, $name, $value)?;
    };
    (@field omitempty, $enc:ident, $first:ident, $name:literal, $value:expr) => {
        marshal_field_omitempty($enc, &mut $first, $name, $value)?;
    };
}

macro_rules! handle_json {
    (@unmarshal uint, $self:ident, $dec:ident) => {{
        $self.0 = json_ext::unmarshal_uint_as($dec, &go_type_name::<Self>())?;
        Ok(())
    }};
    (@unmarshal string, $self:ident, $dec:ident) => {
        json_ext::unmarshal_string_as($dec, &mut $self.0, &go_type_name::<Self>())
    };
    ($kind:ident: $($name:ident),*) => {$(
        impl MarshalerTo for $name {
            fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
                self.0.marshal_json_to(enc)
            }
        }

        impl UnmarshalerFrom for $name {
            fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError> {
                handle_json!(@unmarshal $kind, self, dec)
            }
        }

        impl IsZero for $name {
            fn is_zero(&self) -> bool {
                self.0.is_zero()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
    )*};
}

macro_rules! core_int_json {
    ($($ty:ident: $go:literal),* $(,)?) => {$(
        impl MarshalerTo for $ty {
            fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
                self.0.marshal_json_to(enc)
            }
        }

        impl UnmarshalerFrom for $ty {
            fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError> {
                self.0
                    .unmarshal_json_from(dec)
                    .map_err(|err| match SemanticError::of(&err) {
                        Some(mut s) => {
                            s.go_type = $go.to_string();
                            s.into_json_error()
                        }
                        None => err,
                    })
            }
        }

        impl IsZero for $ty {
            fn is_zero(&self) -> bool {
                self.0 == 0
            }
        }
    )*};
}

#[derive(Default, Debug)]
pub struct InitResponse { pub version: u32, pub name: String, pub ready: bool }
proto_json!(both InitResponse { version: "version" plain, name: "name" omitempty, ready: "ready" omitzero });

#[derive(Default, Debug)]
pub struct NodeHandle(pub u32);
#[derive(Default, Debug)]
pub struct NameHandle(pub String);
handle_json!(uint: NodeHandle);
handle_json!(string: NameHandle);

#[derive(Default, Debug)]
pub struct ModuleKind(pub u32);
core_int_json!(ModuleKind: "core.ModuleKind");

fn main() {
    let r = InitResponse { version: 2, name: "tsgo".into(), ready: false };
    let mut enc = String::new();
    r.marshal_json_to(&mut enc).unwrap();
    println!("marshal={enc}"); // ready omitted: it is zero
    let mut back = InitResponse::default();
    let mut dec = JsonDecoder { fields: &["version", "name", "mystery"], uint: 7, text: "hi" };
    back.unmarshal_json_from(&mut dec).unwrap();
    println!("unmarshal={back:?}");
    let h = NodeHandle(12);
    let mut enc = String::new(); h.marshal_json_to(&mut enc).unwrap();
    println!("handle={enc} display={h} zero={} name_zero={}", h.is_zero(), NameHandle::default().is_zero());
    let mut mk = ModuleKind(3);
    let mut dec = JsonDecoder { fields: &[], uint: 5, text: "" };
    mk.unmarshal_json_from(&mut dec).unwrap();
    println!("modulekind={} zero={}", mk.0, mk.is_zero());
}
