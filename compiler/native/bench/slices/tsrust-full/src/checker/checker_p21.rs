//! Port of `checker/checker.go` lines 18536-19435: optionality helpers,
//! cached node/modifier flags, the type resolution stack, property, signature
//! and index info lookup, structured member resolution, base types, and
//! signature instantiation.

use crate::prelude::*;
use std::borrow::Cow;



// Go: checker/checker.go:19495 findIndexInfo
// PORT: package-level Go function that reads index info data, so it is a
// `Checker` method (`&self`).


/// The slots of `Checker::augment_filters`: Go `globalFunctionType`,
/// `globalCallableFunctionType`, `globalNewableFunctionType` and
/// `globalObjectType`.
pub const AUGMENT_FUNCTION: usize = 0;
pub const AUGMENT_CALLABLE: usize = 1;
pub const AUGMENT_NEWABLE: usize = 2;
pub const AUGMENT_OBJECT: usize = 3;

/// The two bits of `name` in an `AugmentFilters` filter, from its id.
/// Equal texts have one id.
#[inline]
fn augment_filter_bits(name: &Name) -> (usize, usize) {
    let mixed = name.id().wrapping_mul(0x9E37_79B9);
    ((mixed >> 24) as usize, ((mixed >> 16) & 255) as usize)
}

/// False when `filter` rejects the name with `bits`.
#[inline]
fn augment_filter_has(filter: &[u64; 4], (a, b): (usize, usize)) -> bool {
    filter[a >> 6] & (1 << (a & 63)) != 0 && filter[b >> 6] & (1 << (b & 63)) != 0
}

#[cfg(test)]
mod augment_filter_tests {
    use super::*;

    /// The types of the type aliases of `a.ts`, in source order. Each
    /// scenario declares the same 10 aliases, so every lookup runs on a
    /// plain object, a callable, a newable, a type with both signatures,
    /// a class constructor, `Function`, `Object`, an array, `{}` and a
    /// primitive (its apparent type).
    const ALIASES: &str = r#"
type T0 = { a: number };
type T1 = () => void;
type T2 = new () => {};
type T3 = { (): void; new (): {} };
type T4 = typeof K;
type T5 = Function;
type T6 = Object;
type T7 = string[];
type T8 = {};
type T9 = string;
"#;

    /// Loads `files` with the `compilerOptions` body `options`, checks
    /// `a.ts`, and calls `f` on its checker with the codes of its
    /// diagnostics and the types of its type aliases. Each call writes its
    /// files in a temp dir of its own.
    fn with_checked_a<R: Send + 'static>(
        files: &[(&str, String)],
        options: &str,
        f: impl FnOnce(&mut Checker, Vec<i32>, Vec<TypeId>) -> R + Send + 'static,
    ) -> R {
        static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let call = CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("ts_goport_augfilter_{}_{call}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (name, text) in files {
            std::fs::write(dir.join(name), text).unwrap();
        }
        let list: Vec<String> = files.iter().map(|(name, _)| format!("{name:?}")).collect();
        std::fs::write(
            dir.join("tsconfig.json"),
            format!(
                r#"{{ "compilerOptions": {{ {options} }}, "files": [{}] }}"#,
                list.join(", ")
            ),
        )
        .unwrap();
        let config = dir.join("tsconfig.json");
        let program = crate::program::try_load_version(&config.to_string_lossy(), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {}: {e}", config.display()));
        let _ = std::fs::remove_dir_all(&dir);
        let scope = crate::core::enter_program(Some(program));
        let file = program
            .source_files()
            .find(|file| file.info.file_name.ends_with("/a.ts"))
            .expect("a.ts is not in the program")
            .root;
        let result = crate::program::with_type_checker_for_file(file, move |checker| {
            let ctx = crate::gostd::context::background();
            let codes = checker
                .get_diagnostics_exported(&ctx, file)
                .iter()
                .map(|d| d.code)
                .collect();
            let types = file
                .statements()
                .iter()
                .filter(|s| s.kind() == SyntaxKind::TypeAliasDeclaration)
                .map(|alias| checker.get_type_from_type_node(alias.type_()))
                .collect();
            f(checker, codes, types)
        });
        drop(scope);
        crate::program::release_program(program);
        result
    }

    /// Go `getPropertyOfTypeEx(t, name, false, false)` for a type whose
    /// apparent type is an object type, with the two lookups of
    /// checker.go:19259-19274 always made.
    fn go_property_of_type(c: &mut Checker, t: TypeId, name: &Name) -> SymbolId {
        let own = c.get_property_of_type_ex(t, name, true, false);
        if own.is_some() {
            return own;
        }
        let t = c.get_reduced_apparent_type(t);
        let resolved = c.resolve_structured_type_members(t);
        let (calls, constructs) = (
            resolved.call_signatures().len(),
            resolved.construct_signatures().len(),
        );
        let function_type = if t == c.any_function_type {
            c.global_function_type
        } else if calls != 0 {
            c.global_callable_function_type
        } else if constructs != 0 {
            c.global_newable_function_type
        } else {
            TypeId::NIL
        };
        if function_type.is_some() {
            let symbol = c.get_property_of_object_type_key(function_type, TableKey::Name(name));
            if symbol.is_some() {
                return symbol;
            }
        }
        let object = c.global_object_type;
        c.get_property_of_object_type_key(object, TableKey::Name(name))
    }

    /// The names to look up: the members of the 4 filter types, every
    /// global and the members and exports of each, `extra`, and 256 names
    /// that no declaration has.
    fn names_to_try(c: &Checker, extra: &[&str]) -> Vec<Name> {
        let mut seen = FxHashSet::default();
        let mut names = Vec::new();
        let mut add = |name: Name| {
            if seen.insert(name.id()) {
                names.push(name);
            }
        };
        for g in [
            c.global_function_type,
            c.global_callable_function_type,
            c.global_newable_function_type,
            c.global_object_type,
        ] {
            for (name, _) in c.symbols.entries(c.ty(g).as_structured_type().members) {
                add(name);
            }
        }
        for (name, symbol) in c.symbols.entries(c.globals) {
            add(name);
            for table in [c.sym(symbol).members, c.sym(symbol).exports] {
                for (member, _) in c.symbols.entries(table) {
                    add(member);
                }
            }
        }
        for &text in extra {
            add(Name::from(text));
        }
        for i in 0..256 {
            add(Name::from(format!("noSuchMember{i}")));
        }
        names
    }

    /// True when each slot of `augment_filters` holds the filter of its
    /// type, and the union is built.
    fn all_filters_built(c: &Checker) -> bool {
        c.augment_filters.of == c.augment_globals() && c.augment_filters.all.is_some()
    }

    /// Resolves the 4 filter types by lookups that Go makes too, then
    /// misses once on each kind of type, so the 4 filters are built.
    fn build_filter(c: &mut Checker, types: &[TypeId]) {
        let (plain, callable, newable) = (types[0], types[1], types[2]);
        let miss = Name::from("noSuchMember0");
        for t in [callable, newable, c.any_function_type, plain] {
            c.get_property_of_type(t, "toString");
            assert!(c.get_property_of_type_ex(t, &miss, false, false).is_nil());
        }
        assert!(all_filters_built(c), "the filters are not built");
    }

    /// Every name on every alias type (and on Go's any function type) gives
    /// Go's symbol. Returns how many names the filters reject on a
    /// callable, so a test can check that it skipped lookups.
    fn assert_lookups_match_go(c: &mut Checker, types: &[TypeId], extra: &[&str]) -> usize {
        build_filter(c, types);
        let names = names_to_try(c, extra);
        let mut all = types.to_vec();
        all.push(c.any_function_type);
        for &t in &all {
            let apparent = c.get_reduced_apparent_type(t);
            assert!(c.ty(apparent).flags.intersects(TypeFlags::OBJECT));
            for name in &names {
                let want = go_property_of_type(c, t, name);
                let got = c.get_property_of_type_ex(t, name, false, false);
                assert_eq!(got, want, "{:?} on type {t:?}", name.as_str());
            }
        }
        assert!(all_filters_built(c), "a filter was dropped");
        names
            .iter()
            .filter(|name| c.augment_lookups_miss(types[1], name))
            .count()
    }

    /// True when `name` on `t` is found and declared in `a.ts` or `b.ts`.
    fn found_in_test_file(c: &mut Checker, t: TypeId, name: &str) -> bool {
        let name = Name::from(name);
        let symbol = c.get_property_of_type_ex(t, &name, false, false);
        symbol.is_some()
            && c.sym(symbol).declarations.iter().any(|d| {
                let file_name = source_file_file_name(get_source_file_of_node(*d));
                file_name.ends_with("/a.ts") || file_name.ends_with("/b.ts")
            })
    }

    const STRICT: &str = r#""strict": true, "target": "es2020", "types": []"#;

    /// `declare global` adds a member to each of the 4 types. The filter
    /// holds them, so a plain object, a callable and a newable find them
    /// through the fallback.
    #[test]
    fn filter_keeps_global_augmentations() {
        let a = format!(
            r#"export {{}};
declare global {{
    interface Object {{ objAug(): void }}
    interface Function {{ fnAug: number }}
    interface CallableFunction {{ callAug: string }}
    interface NewableFunction {{ newAug: boolean }}
}}
declare const plain: {{ a: number }};
declare const call: () => void;
declare const ctor: new () => {{}};
declare const both: {{ (): void; new (): {{}} }};
declare class K {{ static s: number }}
declare const u: {{ a: number }} | {{ b: number }};
call.apply; ctor.bind; plain.toString;
if ("a" in u) {{}}
plain.objAug(); call.objAug(); call.fnAug; call.callAug;
ctor.newAug; ctor.fnAug; ctor.objAug(); both.callAug; K.newAug; K.fnAug;
plain.absentMember;
{ALIASES}"#
        );
        let (codes, rejected, found) = with_checked_a(&[("a.ts", a)], STRICT, |c, codes, types| {
            let found = [
                (types[0], "objAug"),
                (types[1], "objAug"),
                (types[1], "fnAug"),
                (types[1], "callAug"),
                (types[2], "newAug"),
                (types[2], "fnAug"),
                (types[4], "newAug"),
                (types[5], "objAug"),
                (types[9], "objAug"),
            ]
            .map(|(t, name)| found_in_test_file(c, t, name));
            let extra = ["objAug", "fnAug", "callAug", "newAug", "absentMember"];
            (codes, assert_lookups_match_go(c, &types, &extra), found)
        });
        // Go N gives only TS2339 for `absentMember`.
        assert_eq!(codes, [2339]);
        assert_eq!(found, [true; 9]);
        assert!(rejected > 128, "the filter rejected {rejected} names");
    }

    /// A script merges members into the global interfaces, also with late
    /// bound names (`[fnKey]`, `[objKey]`). The filter holds the merged
    /// tables.
    #[test]
    fn filter_keeps_merged_and_late_bound_members() {
        let b = r#"interface Object { scriptObj: number }
interface Function { scriptFn(): void }
declare const fnKey: unique symbol;
declare const objKey: unique symbol;
interface Function { [fnKey]: number }
interface Object { [objKey]: string }
"#;
        let a = format!(
            r#"export {{}};
declare const plain: {{ a: number }};
declare const call: () => void;
declare const ctor: new () => {{}};
declare class K {{ static s: number }}
call.apply; ctor.bind; plain.toString;
const n: number = call[fnKey];
const m: number = ctor[fnKey];
const s: string = plain[objKey];
const t: string = call[objKey];
plain.scriptObj; call.scriptFn(); call.scriptObj; K.scriptFn();
{ALIASES}"#
        );
        let (codes, rejected, found) = with_checked_a(
            &[("a.ts", a), ("b.ts", b.to_string())],
            STRICT,
            |c, codes, types| {
                let found = [
                    (types[0], "scriptObj"),
                    (types[1], "scriptFn"),
                    (types[1], "scriptObj"),
                    (types[2], "scriptFn"),
                ]
                .map(|(t, name)| found_in_test_file(c, t, name));
                let extra = ["scriptObj", "scriptFn"];
                (codes, assert_lookups_match_go(c, &types, &extra), found)
            },
        );
        assert_eq!(codes, Vec::<i32>::new());
        assert_eq!(found, [true; 4]);
        assert!(rejected > 128, "the filter rejected {rejected} names");
    }

    /// With `strictBindCallApply` off, CallableFunction and NewableFunction
    /// are the Function type, and the es5 lib has no `Symbol` members.
    #[test]
    fn filter_follows_lib_es5_without_strict_bind_call_apply() {
        let a = format!(
            r#"export {{}};
declare global {{
    interface Function {{ fnAug: number }}
}}
declare const plain: {{ a: number }};
declare const call: () => void;
declare const ctor: new () => {{}};
declare class K {{ static s: number }}
call.apply; ctor.bind; plain.toString;
call.fnAug; ctor.fnAug; K.fnAug; plain.fnAug;
{ALIASES}"#
        );
        let options = r#""strict": true, "strictBindCallApply": false, "target": "es2015", "lib": ["es5"], "types": []"#;
        let (codes, rejected, same) = with_checked_a(&[("a.ts", a)], options, |c, codes, types| {
            let same = c.global_callable_function_type == c.global_function_type
                && c.global_newable_function_type == c.global_function_type;
            (codes, assert_lookups_match_go(c, &types, &["fnAug"]), same)
        });
        // Go N gives only TS2339 for `plain.fnAug`.
        assert_eq!(codes, [2339]);
        assert!(same);
        assert!(rejected > 128, "the filter rejected {rejected} names");
    }

    /// With `noLib`, the globals come from a file of the project. There
    /// NewableFunction is a type alias, so Go `getGlobalType` gives the
    /// empty object type, which has no members.
    #[test]
    fn filter_follows_no_lib_globals() {
        let globals = r#"interface Object { o1: number }
interface Function { f1: number }
interface CallableFunction extends Function { c1: number }
interface Array<T> { length: number }
interface String {}
interface Boolean {}
interface Number {}
interface RegExp {}
interface IArguments {}
type NewableFunction = { n1: number }
"#;
        let a = format!(
            r#"export {{}};
declare const plain: {{ a: number }};
declare const call: () => void;
declare const ctor: new () => {{}};
declare class K {{ static s: number }}
call.c1; call.f1; call.o1; ctor.o1; plain.o1;
ctor.f1; ctor.n1;
{ALIASES}"#
        );
        let options = r#""strict": true, "noLib": true, "target": "es2020", "types": []"#;
        let (codes, rejected, empty) = with_checked_a(
            &[("a.ts", a), ("globals.d.ts", globals.to_string())],
            options,
            |c, codes, types| {
                let empty = c.global_newable_function_type == c.empty_object_type;
                (
                    codes,
                    assert_lookups_match_go(c, &types, &["o1", "f1", "c1", "n1"]),
                    empty,
                )
            },
        );
        // Go N gives TS2339 for `ctor.f1` and `ctor.n1` (and TS2316 in
        // globals.d.ts).
        assert_eq!(codes, [2339, 2339]);
        assert!(empty);
        assert!(rejected > 128, "the filter rejected {rejected} names");
    }

    /// When the members of Function are set again (the outer resolution
    /// of a nested one, here with a member added to the declared table),
    /// its filter is built again from the new table. The `get_base_types`
    /// reset drops the filter of its type.
    #[test]
    fn filter_follows_members_changes() {
        let a = format!(
            r#"export {{}};
declare class K {{ static s: number }}
{ALIASES}"#
        );
        let (added_found, reset_dropped) = with_checked_a(&[("a.ts", a)], STRICT, |c, _, types| {
            build_filter(c, &types);
            let function = c.global_function_type;
            // A name that the filters reject (ids, and so bits, depend on
            // intern order).
            let any_function = c.any_function_type;
            let added = (0..)
                .map(|i| Name::from(format!("addedLater{i}")))
                .find(|name| c.augment_lookups_miss(any_function, name))
                .unwrap();
            let table = c
                .symbols
                .clone_table(c.ty(function).as_interface_type().declared_members);
            let symbol = c.new_symbol(SymbolFlags::PROPERTY, &added);
            c.symbols.set(table, &added, symbol);
            c.ty_mut(function).as_interface_type_mut().declared_members = table;
            let flags = c
                .ty(function)
                .object_flags
                .without(ObjectFlags::MEMBERS_RESOLVED);
            c.ty_mut(function).object_flags = flags;
            c.resolve_structured_type_members(function);
            assert!(all_filters_built(c));
            assert!(!c.augment_lookups_miss(any_function, &added));
            let added_found =
                c.get_property_of_type_ex(any_function, &added, false, false) == symbol;

            build_filter(c, &types);
            let callable = c.global_callable_function_type;
            c.ty_mut(callable)
                .as_interface_type_mut()
                .base_types_resolved = false;
            c.ty_mut(callable)
                .as_interface_type_mut()
                .resolved_base_types = SharedList::default();
            c.get_base_types(callable);
            let reset_dropped = c.augment_filters.of[AUGMENT_CALLABLE].is_nil()
                && c.augment_filters.all.is_none()
                && !c
                    .ty(callable)
                    .object_flags
                    .intersects(ObjectFlags::MEMBERS_RESOLVED);
            assert_lookups_match_go(c, &types, &[added.as_str()]);
            (added_found, reset_dropped)
        });
        assert!(added_found);
        assert!(reset_dropped);
    }

    /// While NewableFunction is not resolved, misses on a callable and on
    /// a plain object use the filters of the resolved types, with no union,
    /// and build nothing again. A miss on a newable makes Go's lookups,
    /// which resolve NewableFunction and so build only its filter, and the
    /// union.
    #[test]
    fn filter_of_each_type_is_built_once() {
        let a = r#"export {};
type T0 = { a: number };
type T1 = () => void;
type T2 = new () => {};
"#;
        with_checked_a(&[("a.ts", a.to_string())], STRICT, |c, _, types| {
            let (plain, callable, newable) = (types[0], types[1], types[2]);
            let newable_function = c.global_newable_function_type;
            let resolved = |c: &Checker| {
                c.ty(newable_function)
                    .object_flags
                    .intersects(ObjectFlags::MEMBERS_RESOLVED)
            };
            assert!(!resolved(c), "NewableFunction is resolved too early");
            let miss = Name::from("noSuchMember0");
            for t in [callable, plain] {
                c.get_property_of_type(t, "toString");
                assert!(c.get_property_of_type_ex(t, &miss, false, false).is_nil());
            }
            let of = c.augment_filters.of;
            assert_eq!(of[AUGMENT_CALLABLE], c.global_callable_function_type);
            assert_eq!(of[AUGMENT_OBJECT], c.global_object_type);
            assert!(of[AUGMENT_NEWABLE].is_nil());
            assert!(c.augment_filters.all.is_none());
            // A new build would write the real bits over these.
            for slot in [AUGMENT_CALLABLE, AUGMENT_OBJECT] {
                c.augment_filters.bits[slot] = [!0; 4];
            }
            for i in 0..64 {
                let miss = Name::from(format!("noSuchMember{i}"));
                for t in [plain, callable] {
                    assert!(c.get_property_of_type_ex(t, &miss, false, false).is_nil());
                }
            }
            assert!(!resolved(c));
            assert!(
                c.get_property_of_type_ex(newable, &miss, false, false)
                    .is_nil()
            );
            assert!(
                resolved(c),
                "the newable miss did not resolve NewableFunction"
            );
            assert!(all_filters_built(c));
            for slot in [AUGMENT_CALLABLE, AUGMENT_OBJECT] {
                assert_eq!(
                    c.augment_filters.bits[slot], [!0; 4],
                    "slot {slot} was built again"
                );
            }
        });
    }

    /// The union waits for all 4 types (R183 reviewer item 2).
    /// `initialize_checker` sets Object before Function, CallableFunction
    /// and NewableFunction (Go checker.go:1361-1364). A filter of Object
    /// built between them must not make a union of Object's names alone:
    /// once Function is set, the lookups of its names would skip it.
    #[test]
    fn union_waits_for_all_4_types() {
        let a = r#"export {};
type T0 = { a: number };
type T1 = () => void;
type T2 = new () => {};
"#;
        let (early_union, skipped) =
            with_checked_a(&[("a.ts", a.to_string())], STRICT, |c, _, types| {
                build_filter(c, &types);
                let globals = c.augment_globals();
                c.global_function_type = TypeId::NIL;
                c.global_callable_function_type = TypeId::NIL;
                c.global_newable_function_type = TypeId::NIL;
                c.augment_filters = AugmentFilters::default();
                c.build_augment_filter(globals[AUGMENT_OBJECT]);
                let early_union = c.augment_filters.all.is_some();
                c.global_function_type = globals[AUGMENT_FUNCTION];
                c.global_callable_function_type = globals[AUGMENT_CALLABLE];
                c.global_newable_function_type = globals[AUGMENT_NEWABLE];
                let function = c.ty(globals[AUGMENT_FUNCTION]).as_structured_type().members;
                let names: Vec<Name> = c
                    .symbols
                    .iter_names(function)
                    .map(|(name, _)| name)
                    .collect();
                let any_function = c.any_function_type;
                let skipped: Vec<String> = names
                    .iter()
                    .filter(|name| c.augment_lookups_miss(any_function, name))
                    .map(|name| name.as_str().to_string())
                    .collect();
                assert_lookups_match_go(c, &types, &[]);
                (early_union, skipped)
            });
        assert!(!early_union, "a union of Object's names alone");
        assert_eq!(skipped, Vec::<String>::new());
    }

    /// The source of 40 names `{prefix}0` to `{prefix}39` that a module
    /// augmentation adds to `iface`, and of an assignment of `value` to a
    /// type with each name, which needs the name on the apparent type of
    /// `value`.
    fn late_members(iface: &str, prefix: &str, value: &str) -> (String, String) {
        let decls = (0..40)
            .map(|i| format!("  interface {iface} {{ {prefix}{i}: number }}\n"))
            .collect();
        let uses = (0..40)
            .map(|i| format!("const s{i}: {{ {prefix}{i}: number }} = {value};\n"))
            .collect();
        (decls, uses)
    }

    /// Checks `a.ts` of `files` with `options` and the commonjs module.
    /// Gives the codes of `a.ts`, the types that the 4 slots hold, the 4
    /// types, and whether the union is built.
    fn check_augmentation(
        files: &[(&str, String)],
        options: &str,
    ) -> (Vec<i32>, [TypeId; 4], [TypeId; 4], bool) {
        let options = format!(r#"{options}, "module": "commonjs""#);
        with_checked_a(files, &options, |c, codes, _| {
            let filters = c.augment_filters;
            (
                codes,
                filters.of,
                c.augment_globals(),
                filters.all.is_some(),
            )
        })
    }

    /// `x.ts` with an `export=` whose check resolves Object (and with
    /// `calls`, the function types), so their filters are built in the
    /// module augmentation loop of `initialize_checker`.
    fn x_ts(calls: bool) -> (&'static str, String) {
        let text = if calls {
            r#"declare function f(): void;
declare class C {}
declare const p: { a: number };
const o = { r1: f.call, r2: C.apply, r3: p.toString, r4: p.missingX };
export = o.r1;
"#
        } else {
            r#"declare const p: { a: number };
const o = { r3: p.toString };
export = o.r3;
"#
        };
        ("x.ts", text.to_string())
    }

    fn g_ts() -> (&'static str, String) {
        ("g.ts", "export = globalThis;\n".to_string())
    }

    /// The augmentation of `./x` resolves Object, then the augmentation of
    /// `./g` (`export = globalThis`) adds 3 names to the members of the
    /// merged Object in place (Go `mergeSymbol`, checker.go:1484 and
    /// :14414). The filter is built again with them.
    #[test]
    fn filter_follows_augmentation_merged_into_object() {
        let a = r#"import "./x";
import "./g";
declare global { interface Object { a1: number } }
declare module "./x" { interface Q {} }
declare module "./g" { interface Object { late1: number; late2: number; late3: number } }
declare const p2: { b: number };
const s1: { late1: number } = p2;
const s2 = p2["late2"];
const s3: { late3: number } = p2;
"#;
        let files = [("a.ts", a.to_string()), x_ts(false), g_ts()];
        let (codes, of, globals, _) = check_augmentation(&files, STRICT);
        // Go N gives only TS2671 for `./x`.
        assert_eq!(codes, [2671]);
        assert_eq!(of[AUGMENT_OBJECT], globals[AUGMENT_OBJECT]);
    }

    /// As above with 40 names, after the check of `./x` resolved all 4
    /// types, so lookups test the union.
    #[test]
    fn union_follows_augmentation_merged_into_object() {
        let (decls, uses) = late_members("Object", "zq", "p2");
        let reads: Vec<String> = (0..40).map(|i| format!("p2[\"zq{i}\"]")).collect();
        let a = format!(
            r#"import "./x";
import "./g";
declare global {{
  interface Object {{ a1: number }}
  interface Function {{ f1: number }}
}}
declare module "./x" {{ interface Q {{ q: number }} }}
declare module "./g" {{
{decls}}}
declare const p2: {{ b: number }};
export const r = [{}];
{uses}"#,
            reads.join(", ")
        );
        let files = [("a.ts", a), x_ts(true), g_ts()];
        let (codes, of, globals, all) = check_augmentation(&files, STRICT);
        // Go N gives only TS2671 for `./x` (and TS2339 in `x.ts`).
        assert_eq!(codes, [2671]);
        assert_eq!(of, globals);
        assert!(all);
    }

    /// As above when only Object is resolved, so lookups test its filter
    /// alone.
    #[test]
    fn object_filter_follows_augmentation_merged_into_object() {
        let (decls, uses) = late_members("Object", "zq", "p2");
        let a = format!(
            r#"import "./x";
import "./g";
declare global {{ interface Object {{ a1: number }} }}
declare module "./x" {{ interface Q {{ q: number }} }}
declare module "./g" {{
{decls}}}
declare const p2: {{ b: number }};
{uses}"#
        );
        let files = [("a.ts", a), x_ts(false), g_ts()];
        let (codes, of, globals, all) = check_augmentation(&files, STRICT);
        // Go N gives only TS2671 for `./x`.
        assert_eq!(codes, [2671]);
        assert_eq!(of[AUGMENT_OBJECT], globals[AUGMENT_OBJECT]);
        assert!(!all);
    }

    /// The augmentation adds 40 names to Function. With the es5 lib and
    /// `strictBindCallApply` off, the 3 function slots are all Function.
    #[test]
    fn function_filter_follows_augmentation_merged_into_function() {
        let (decls, uses) = late_members("Function", "fq", "f2");
        let a = format!(
            r#"import "./x";
import "./g";
declare global {{ interface Function {{ f1: number }} }}
declare module "./x" {{ interface Q {{ q: number }} }}
declare module "./g" {{
{decls}}}
declare function f2(): void;
{uses}"#
        );
        let (_, x) = x_ts(true);
        let x = x.replace("export =", "const t1: { fx1: number } = f;\nexport =");
        let files = [("a.ts", a), ("x.ts", x), g_ts()];
        let options = r#""strict": true, "strictBindCallApply": false, "target": "es2015", "lib": ["es5"], "types": []"#;
        let (codes, of, globals, _) = check_augmentation(&files, options);
        // Go N gives only TS2671 for `./x` (and TS2322 and TS2339 in
        // `x.ts`).
        assert_eq!(codes, [2671]);
        assert_eq!(globals[..3], [globals[AUGMENT_FUNCTION]; 3]);
        assert_eq!(of[..3], globals[..3]);
    }

    /// The augmented module is ambient: `declare module "glob" { export =
    /// globalThis; }`.
    #[test]
    fn filter_follows_augmentation_of_ambient_module() {
        let (decls, uses) = late_members("Object", "zq", "p2");
        let a = format!(
            r#"import "./x";
import "glob";
declare global {{ interface Object {{ a1: number }} }}
declare module "./x" {{ interface Q {{ q: number }} }}
declare module "glob" {{
{decls}}}
declare const p2: {{ b: number }};
{uses}"#
        );
        let ambient = r#"declare module "glob" { export = globalThis; }"#;
        let files = [("amb.d.ts", ambient.to_string()), ("a.ts", a), x_ts(false)];
        let (codes, of, globals, _) = check_augmentation(&files, STRICT);
        // Go N gives only TS2671 for `./x`.
        assert_eq!(codes, [2671]);
        assert_eq!(of[AUGMENT_OBJECT], globals[AUGMENT_OBJECT]);
    }

    /// Skeptic program e25 (propfilt1-skeptic-c, R183 reviewer item 3).
    /// The merge of `./x` resolves Object (its `export=` reads
    /// `p.toString`), the merge of `./g` (`export = globalThis`) adds
    /// `late1` and `late2` to the members of Object in place, and the merge
    /// of `./y` then reads them. So the filter must be built again after
    /// each merge, not once after the loop (mutant M2).
    #[test]
    fn filter_follows_each_module_augmentation_merge() {
        let a = r#"import "./x";
import "./y";
import "./g";
declare global { interface Object { a1: number } }
declare module "./x" { interface Q {} }
declare module "./g" { interface Object { late1: number; late2: number } }
declare module "./y" { interface R {} }
export {};
"#;
        let x = r#"declare const p: { a: number };
const o = { s: p.toString };
export = o.s;
"#;
        let y = r#"declare const q: { a: number };
const o = { s: q["late1"], t: q satisfies { late2: number }, u: q["missingY"] };
export = o.s;
"#;
        let files = [
            ("a.ts", a.to_string()),
            ("x.ts", x.to_string()),
            ("y.ts", y.to_string()),
            g_ts(),
        ];
        let options = format!(r#"{STRICT}, "module": "commonjs""#);
        let (a_codes, y_codes) = with_checked_a(&files, &options, |c, codes, _| {
            let y = c
                .files
                .iter()
                .copied()
                .find(|&file| source_file_file_name(file).ends_with("/y.ts"))
                .expect("y.ts is not in the program");
            let ctx = crate::gostd::context::background();
            let y_codes: Vec<i32> = c
                .get_diagnostics_exported(&ctx, y)
                .iter()
                .map(|d| d.code)
                .collect();
            (codes, y_codes)
        });
        // Go N (tsgo-oracle-673a5f17d713): TS2671 for `./x` and `./y`, and
        // in `y.ts` only TS7053 for `missingY`.
        assert_eq!(a_codes, [2671, 2671]);
        assert_eq!(y_codes, [7053]);
    }
}
